use std::sync::Arc;

use api::{GameAction, GameError, GameEvent, GameId, UserId, WsResponse};
use hexomino_core::{Action, Player, State};
use parking_lot::Mutex;
use uuid::Uuid;

use crate::result::ApiResult;

use super::user::User;

type Result<T> = ApiResult<T, GameError>;

#[derive(Clone, Debug, derive_more::Deref)]
pub struct Game(Arc<GameInner>);

impl Game {
    pub fn new(u1: User, u2: User) -> Self {
        Self(Arc::new(GameInner {
            id: GameId(Uuid::new_v4()),
            state: Mutex::new(GameState::new()),
            users: [u1, u2],
        }))
    }

    pub fn id(&self) -> GameId {
        self.id
    }
}

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct GameInner {
    id: GameId,
    #[derivative(Debug="ignore")]
    state: Mutex<GameState>,
    users: [User; 2],
}

struct GameState {
    game: State,
    players: [PlayerState; 2],
}

impl GameState {
    fn new() -> Self {
        Self {
            game: State::new(),
            players: [PlayerState::new(), PlayerState::new()],
        }
    }
}

struct PlayerState {
    is_connected: bool,
}

impl PlayerState {
    fn new() -> Self {
        Self {
            is_connected: false,
        }
    }
}

impl GameInner {
    pub fn user_action(&self, user_id: UserId, action: GameAction) -> Result<()> {
        tracing::debug!("get action = {action:?}");
        let player = self.get_player(user_id).ok_or(GameError::NotInGame)?;
        let mut state = self.state.lock();
        let pid = player.id();
        match action {
            GameAction::Connected => {
                state.players[pid].is_connected = true;
            }
            GameAction::Play(action) => {
                if state.game.current_player() != Some(player) {
                    return Err(GameError::NotYourTurn.into());
                }
                state
                    .game
                    .play(action)
                    .map_err(|err| GameError::GameActionError(format!("{err}")))?;
                self.broadcast_action(action);
            }
        }
        Ok(())
    }
}

impl GameInner {
    fn get_player(&self, user_id: UserId) -> Option<Player> {
        if self.users[0].id() == user_id {
            Some(Player::First)
        } else if self.users[1].id() == user_id {
            Some(Player::Second)
        } else {
            None
        }
    }

    fn broadcast_action(&self, action: Action) {
        tracing::debug!("broadcast action");
        for users in &self.users {
            users.spawn_send(WsResponse::GameEvent(GameEvent::UserPlay(action)));
        }
    }
}
