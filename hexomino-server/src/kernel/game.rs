use std::{rc::Rc, sync::Arc, time::Duration};

use anyhow::Context as _;
use api::{
    GameEndReason, GameInnerState, MatchAction, MatchEndInfo, MatchError, MatchEvent, MatchId,
    MatchInnerState, MatchWinner, UserId, UserPlay, WsResponse,
};
use getset::CopyGetters;
use hexomino_core::{Action, Player, State as GameState};
use uuid::Uuid;

use crate::result::ApiResult;

use super::{
    actor::{Actor, Addr, Context, Handler},
    user::{User, UserData, UserStatus},
};

type Result<T> = ApiResult<T, MatchError>;

#[derive(Clone, derivative::Derivative)]
#[derivative(Debug)]
#[derive(CopyGetters)]
pub struct MatchHandle {
    #[getset(get = "pub")]
    info: Arc<MatchInfo>,
    #[derivative(Debug = "ignore")]
    addr: Addr<MatchActor>,
}

impl MatchHandle {
    pub async fn user_action(&self, user: User, action: MatchAction) -> Result<()> {
        self.addr.send(UserAction { user, action }).await
    }
    pub async fn sync_match(&self, user: User) -> Result<api::MatchState> {
        self.addr.send(SyncMatch { user }).await
    }
}

// TODO: derive Debug for state
pub struct MatchActor {
    info: Arc<MatchInfo>,
    state: MatchState,
    users: [User; 2],
}

#[derive(Debug)]
struct MatchInfo {
    id: MatchId,
    num_games: u32,
    user_data: [api::User; 2],
}

struct MatchState {
    phase: MatchPhase,
    player_states: [PlayerState; 2],
    game_idx: i32,
    game: GameState,
    first_user_player: Player,
    prev_actions: Vec<Action>,
    prev_end_state: Option<GameEndState>,
}

#[derive(Copy, Clone)]
struct GameEndState {
    winner: Player,
    reason: GameEndReason,
}

struct PlayerState {
    is_ready: bool,
    score: u32,
}

#[derive(PartialEq, Eq)]
enum MatchPhase {
    GameNotStarted,
    GamePlaying,
    GameEnded,
    MatchEnded,
}

impl MatchActor {
    pub fn new(users: [User; 2]) -> Self {
        Self {
            info: Arc::new(MatchInfo::new(&users)),
            users,
            state: MatchState::new(),
        }
    }

    pub fn start(self) -> MatchHandle {
        let info = self.info.clone();
        let addr = Actor::start(self);
        MatchHandle { info, addr }
    }

    fn broadcast_last_action(&self) {
        tracing::debug!("broadcast action");
        if let Some(action) = self.state.prev_actions.last().cloned() {
            for users in &self.users {
                users.do_send(WsResponse::MatchEvent(MatchEvent::UserPlay(UserPlay {
                    action,
                    idx: (self.state.prev_actions.len() - 1) as u32,
                })));
            }
        }
    }

    fn broadcast_new_game(&self) {
        let gen_resp = |player| WsResponse::MatchEvent(MatchEvent::GameStart { you: player });
        self.users[0].do_send(gen_resp(self.state.first_user_player));
        self.users[1].do_send(gen_resp(self.state.first_user_player.other()));
    }

    fn broadcast_game_end(&self) {
        let end_state = self.state.prev_end_state.expect("game is not ended");
        for (idx, users) in self.users.iter().enumerate() {
            let info = api::GameEndInfo {
                end_state: api::GameEndState {
                    winner: end_state.winner,
                    reason: end_state.reason,
                },
                scores: self.state.scores(),
            }
            .as_perspective(idx);
            users.do_send(WsResponse::MatchEvent(MatchEvent::GameEnd(info)));
        }
    }

    fn broadcast_match_end(&self) {
        let scores = self.state.scores();
        for (idx, user) in self.users.iter().enumerate() {
            let winner = self.state.winner_from_user(idx);
            let info = MatchEndInfo { scores, winner }.as_perspective(idx);
            user.do_send(WsResponse::MatchEvent(MatchEvent::MatchEnd(info)));
        }
    }

    fn user_idx(&self, user_id: UserId) -> Option<usize> {
        if user_id == self.users[0].id() {
            Some(0)
        } else if user_id == self.users[1].id() {
            Some(1)
        } else {
            None
        }
    }

    fn user_player(&self, user_id: UserId) -> Option<Player> {
        match self.user_idx(user_id) {
            Some(0) => Some(self.state.first_user_player),
            Some(1) => Some(self.state.first_user_player.other()),
            _ => None,
        }
    }

    fn player_to_user_idx(&self, player: Player) -> usize {
        if player == self.state.first_user_player {
            0
        } else {
            1
        }
    }

    fn check_all_ready(&self, ctx: &Context<Self>) {
        if self.state.player_states.iter().all(|p| p.is_ready) {
            tracing::debug!("notify startnewgame");
            ctx.notify(StartNewGame);
        }
    }

    fn player_win_game(&mut self, player: Player, reason: GameEndReason, ctx: &Context<Self>) {
        let user_idx = self.player_to_user_idx(player);
        let state = &mut self.state;
        let score = &mut state.player_states[user_idx].score;
        *score += 1;
        let match_is_end = *score > self.info.num_games / 2;
        state.phase = if match_is_end {
            MatchPhase::MatchEnded
        } else {
            MatchPhase::GameEnded
        };
        self.state.prev_end_state = Some(GameEndState {
            winner: player,
            reason,
        });
        self.broadcast_game_end();

        if !match_is_end {
            ctx.notify_later(StartNewGame, Duration::from_secs(3));
        } else {
            ctx.notify_later(EndMatch, Duration::from_secs(3));
        }
    }
}

impl Actor for MatchActor {}

pub struct SyncMatch {
    user: User,
}

impl Handler<SyncMatch> for MatchActor {
    type Output = Result<api::MatchState>;

    fn handle(&mut self, msg: SyncMatch, ctx: &Context<Self>) -> Self::Output {
        let user_idx = self.user_idx(msg.user.id()).ok_or(MatchError::NotInMatch)?;
        if !self.state.player_states[user_idx].is_ready {
            self.state.player_states[user_idx].is_ready = true;
            self.check_all_ready(ctx);
        }

        let match_state = match self.state.phase {
            MatchPhase::GameNotStarted => MatchInnerState::NotStarted,
            MatchPhase::GamePlaying => {
                MatchInnerState::Playing(api::GameState::GamePlaying(GameInnerState {
                    you: self
                        .user_player(msg.user.id())
                        .expect("already asserted user in game"),
                    prev_actions: self.state.prev_actions.clone(),
                }))
            }
            MatchPhase::GameEnded => {
                let info = self
                    .state
                    .prev_end_state
                    .expect("game ended but info is None");

                MatchInnerState::Playing(api::GameState::GameEnded {
                    game_state: GameInnerState {
                        you: self
                            .user_player(msg.user.id())
                            .expect("already asserted user in game"),
                        prev_actions: self.state.prev_actions.clone(),
                    },
                    end_state: api::GameEndState {
                        winner: info.winner,
                        reason: info.reason,
                    },
                })
            }
            MatchPhase::MatchEnded => MatchInnerState::Ended {
                winner: self.state.winner_from_user(user_idx),
            },
        };

        Ok(api::MatchState {
            info: self.info.to_api(),
            game_idx: self.state.game_idx,
            scores: [
                self.state.player_states[0].score,
                self.state.player_states[1].score,
            ],
            state: match_state,
        }
        .as_perspective(user_idx))
    }
}

pub struct UserAction {
    pub user: User,
    pub action: MatchAction,
}

impl Handler<UserAction> for MatchActor {
    type Output = Result<()>;

    fn handle(&mut self, msg: UserAction, ctx: &Context<Self>) -> Self::Output {
        let player = self
            .user_player(msg.user.id())
            .ok_or(MatchError::NotInMatch)?;
        match msg.action {
            MatchAction::Play(action) => {
                self.state.user_play(player, action)?;
                self.broadcast_last_action();

                // TODO: Test code. Delete me!!
                //self.state.game.set_winner(Player::First);

                if let Some(player) = self.state.game.winner() {
                    self.player_win_game(player, GameEndReason::NoValidMove, ctx);
                } else {
                    //panic!("how come??");
                }
            }
        }
        Ok(())
    }
}

struct StartNewGame;

impl Handler<StartNewGame> for MatchActor {
    type Output = ();

    fn handle(&mut self, msg: StartNewGame, ctx: &Context<Self>) -> Self::Output {
        tracing::debug!("start new game...");
        let state = &mut self.state;
        if state.phase != MatchPhase::GameNotStarted && state.phase != MatchPhase::GameEnded {
            return;
        }
        state.phase = MatchPhase::GamePlaying;
        state.game_idx += 1;
        state.game = GameState::new();
        state.first_user_player = if state.game_idx % 2 == 0 {
            Player::First
        } else {
            Player::Second
        };
        state.prev_actions = vec![];
        state.prev_end_state = None;

        // TODO: Test code. Delete me!
        //state.fast_forward_to_place();

        self.broadcast_new_game();
    }
}

struct EndMatch;

impl Handler<EndMatch> for MatchActor {
    type Output = ();

    fn handle(&mut self, msg: EndMatch, ctx: &Context<Self>) -> Self::Output {
        self.broadcast_match_end();
        let user_states = User::lock_both_user_states(self.users.each_ref());
        for mut state in user_states {
            state.status = UserStatus::Idle;
        }
    }
}

impl MatchInfo {
    fn new(users: &[User; 2]) -> Self {
        Self {
            id: MatchId(Uuid::new_v4()),
            num_games: 1,
            user_data: users.each_ref().map(|u| u.to_api()),
        }
    }

    fn to_api(&self) -> api::MatchInfo {
        api::MatchInfo {
            id: self.id,
            num_games: self.num_games,
            user_data: self.user_data.clone(),
        }
    }
}

impl MatchState {
    fn new() -> Self {
        Self {
            phase: MatchPhase::GameNotStarted,
            player_states: [PlayerState::new(), PlayerState::new()],
            game_idx: -1,
            game: GameState::new(),
            first_user_player: Player::First,
            prev_actions: vec![],
            prev_end_state: None,
        }
    }

    fn user_play(&mut self, player: Player, action: Action) -> Result<()> {
        self.game
            .play(player, action)
            .map_err(|err| MatchError::GameActionError(format!("{err}")))?;
        self.prev_actions.push(action);
        Ok(())
    }

    fn scores(&self) -> [u32; 2] {
        [0, 1].map(|idx| self.player_states[idx].score)
    }

    fn winner(&self) -> Option<usize> {
        let scores = self.scores();
        if scores[0] > scores[1] {
            Some(0)
        } else if scores[1] > scores[0] {
            Some(1)
        } else {
            None
        }
    }

    fn winner_from_user(&self, idx: usize) -> MatchWinner {
        match self.winner() {
            None => MatchWinner::Tie,
            Some(jdx) if jdx == idx => MatchWinner::You,
            _ => MatchWinner::They,
        }
    }

    fn fast_forward_to_place(&mut self) {
        for hexo in hexomino_core::Hexo::all_hexos() {
            self.user_play(self.game.current_player().unwrap(), Action::Pick(hexo));
        }
    }
}

impl PlayerState {
    fn new() -> Self {
        Self {
            is_ready: false,
            score: 0,
        }
    }
}

trait AsPerspective: Sized {
    fn flip(self) -> Self;
    fn as_perspective(self, user_idx: usize) -> Self {
        match user_idx {
            0 => self,
            1 => self.flip(),
            _ => {
                panic!("invalid user idx {user_idx}");
            }
        }
    }
}

impl AsPerspective for api::MatchState {
    fn flip(mut self) -> Self {
        self.info = self.info.flip();
        self.scores.swap(0, 1);
        self
    }
}

impl AsPerspective for api::MatchInfo {
    fn flip(mut self) -> Self {
        self.user_data.swap(0, 1);
        self
    }
}

impl AsPerspective for api::GameEndInfo {
    fn flip(mut self) -> Self {
        self.scores.swap(0, 1);
        self
    }
}

impl AsPerspective for api::MatchEndInfo {
    fn flip(mut self) -> Self {
        self.scores.swap(0, 1);
        self
    }
}
