use anyhow::{Context, Result};
use api::{GameEndReason, MatchHistoryNoGames, MatchId, Never, UserId};
use chrono::{DateTime, Utc};
use hexomino_core::{Action, Player};
use uuid::Uuid;

use crate::result::ApiResult;

use super::{user::unwrap_name_or_unnamed, Kernel};

pub struct MatchHistory {
    id: MatchId,
    users: [UserId; 2],
    scores: [u32; 2],
    games: Vec<GameHistory>,
}

struct GameHistory {
    first_user_player: Player,
    actions: Vec<Action>,
    winner: Player,
    end_reason: GameEndReason,
}

impl MatchHistory {
    pub fn new(id: MatchId, users: [UserId; 2]) -> Self {
        Self {
            id,
            users,
            scores: [0, 0],
            games: vec![],
        }
    }

    pub fn add_game(
        &mut self,
        first_user_player: Player,
        actions: Vec<Action>,
        winner: Player,
        end_reason: GameEndReason,
    ) {
        self.games.push(GameHistory {
            first_user_player,
            actions,
            winner,
            end_reason,
        });

        if winner == first_user_player {
            self.scores[0] += 1;
        } else {
            self.scores[1] += 1;
        }
    }

    pub async fn save(self, end_time: DateTime<Utc>) -> Result<()> {
        let mut tx = Kernel::get().db.begin().await?;

        let mut game_ids = vec![];
        for game in self.games {
            let result = sqlx::query!(
                r#"
                INSERT INTO GameHistories(match_id, user_player_is_swapped,
                    winner_is_first_player, actions_json)
                VALUES ($1, $2, $3, $4)
                RETURNING id;
                "#,
                self.id.0,
                game.first_user_player != Player::First,
                game.winner == Player::First,
                serde_json::to_string(&game.actions)?,
            )
            .fetch_one(&mut tx)
            .await?;

            game_ids.push(result.id);
        }

        let users = self.users.map(|u| u.0);
        let scores = self.scores.map(|x| x as i32);

        sqlx::query!(
            r#"
            INSERT INTO MatchHistories(id, users, scores, end_time, game_histories)
            VALUES ($1, $2, $3, $4, $5);
            "#,
            self.id.0,
            users.as_slice(),
            scores.as_slice(),
            end_time,
            &game_ids,
        )
        .execute(&mut tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO UserHistories(user_id, match_id)
            VALUES
                ($1, $3),
                ($2, $3);
            "#,
            self.users[0].0,
            self.users[1].0,
            self.id.0,
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}

pub async fn list_user_match_histories(user: UserId) -> ApiResult<Vec<MatchHistoryNoGames>, Never> {
    struct Record {
        id: Uuid,
        user_is_first: Option<bool>,
        user0: Option<String>,
        user1: Option<String>,
        scores: Vec<i32>,
        end_time: DateTime<Utc>,
    }
    let result = sqlx::query_as!(
        Record,
        r#"
        SELECT mh.id AS id, u0.name AS user0, u1.name AS user1,
            mh.scores AS scores, u0.id = $1 AS user_is_first, mh.end_time AS end_time
        FROM UserHistories
        JOIN MatchHistories mh ON mh.id = UserHistories.match_id
        JOIN Users u0 ON mh.users[1] = u0.id
        JOIN Users u1 ON mh.users[2] = u1.id
        WHERE UserHistories.user_id = $1
        ORDER BY mh.end_time
        LIMIT 50;
        "#,
        user.0,
    )
    .fetch_all(&Kernel::get().db)
    .await
    .context("failed to query DB")?;

    fn convert_record_to_api(record: Record) -> Result<MatchHistoryNoGames> {
        let id = MatchId(record.id);
        let user_is_first = record
            .user_is_first
            .context("user is first is NULL in the record")?;
        let user0 = unwrap_name_or_unnamed(record.user0);
        let user1 = unwrap_name_or_unnamed(record.user1);
        let scores = <[i32; 2]>::try_from(record.scores)
            .map_err(|_| anyhow::anyhow!("failed to covert scores to [u32; 2]"))?
            .map(|x| x as u32);
        let end_time = record.end_time;
        Ok(MatchHistoryNoGames {
            id,
            user_is_first,
            users: [user0, user1],
            scores,
            end_time,
        })
    }

    Ok(result
        .into_iter()
        .map(convert_record_to_api)
        .collect::<Result<Vec<MatchHistoryNoGames>>>()?)
}
