use std::{cmp::Ordering, sync::Arc, time::Duration};

use api::{
    GameEndReason, GameInnerState, MatchAction, MatchConfig, MatchEndInfo, MatchError, MatchEvent,
    MatchId, MatchInnerState, MatchSettings, MatchToken, MatchWinner, UserId, UserPlay,
    WsNotifiedError, WsResponse,
};
use chrono::{DateTime, Utc};
use hexomino_core::{Action, GamePhase, Player, State as GameState};
use tokio::spawn;
use uuid::Uuid;

use crate::result::ApiResult;

use super::{
    actor::{Actor, Addr, Context, Handler},
    match_history::{self, MatchHistory},
    user::{User, UserStatus},
};

type Result<T> = ApiResult<T, MatchError>;

#[derive(Clone, derivative::Derivative)]
#[derivative(Debug)]
pub struct MatchHandle {
    info: Arc<MatchInfo>,
    #[derivative(Debug = "ignore")]
    addr: Addr<MatchActor>,
}

impl MatchHandle {
    pub async fn user_action(&self, user: User, action: MatchAction) -> Result<()> {
        self.addr.send(UserAction { user, action }).await?
    }
    pub async fn sync_match(&self, user: User) -> Result<api::MatchState> {
        self.addr.send(SyncMatch { user }).await?
    }
    pub fn id(&self) -> MatchId {
        self.info.id
    }
}

// TODO: derive Debug for state
pub struct MatchActor {
    info: Arc<MatchInfo>,
    state: MatchState,
    users: [User; 2],
    history: Option<MatchHistory>,
}

#[derive(Debug)]
pub struct MatchInfo {
    id: MatchId,
    settings: MatchSettings,
    match_token: Option<MatchToken>,
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
    deadline: Deadline,
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

const MATCH_START_WAIT_TIME: Duration = Duration::from_secs(1);
const PICK_PHASE_TIME_LIMIT: Duration = Duration::from_secs(15);
const LEEWAY: Duration = Duration::from_secs(2);
const BETWEEN_GAME_DELAY: Duration = Duration::from_secs(10);

impl MatchActor {
    pub fn new(users: [User; 2], config: MatchConfig, match_token: Option<MatchToken>) -> Self {
        let info = MatchInfo::new(&users, config, match_token.clone());
        let history_info = match_history::MatchInfo {
            id: info.id,
            users: info.user_data.clone().map(|u| u.id),
            config,
            match_token,
        };
        let history = MatchHistory::new(history_info);
        Self {
            info: Arc::new(info),
            users,
            state: MatchState::new(),
            history: Some(history),
        }
    }

    pub fn start(self) -> MatchHandle {
        let info = self.info.clone();
        let addr = Actor::start(self);
        MatchHandle { info, addr }
    }

    fn broadcast_last_action(&self) {
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
        let Some(end_state) = self.state.prev_end_state else {
            tracing::error!("prev_end_state is none");
            return;
        };
        for (idx, users) in self.users.iter().enumerate() {
            let info = api::GameEndInfo {
                end_state: api::GameEndState {
                    winner: end_state.winner,
                    reason: end_state.reason,
                },
                scores: self.state.scores(),
            }
            .into_perspective(idx);
            users.do_send(WsResponse::MatchEvent(MatchEvent::GameEnd(info)));
        }
    }

    fn broadcast_match_end(&self) {
        let scores = self.state.scores();
        for (idx, user) in self.users.iter().enumerate() {
            let winner = self.state.winner_from_user(idx);
            let info = MatchEndInfo { scores, winner }.into_perspective(idx);
            user.do_send(WsResponse::MatchEvent(MatchEvent::MatchEnd(info)));
        }
    }

    fn broadcast_deadline(&self) {
        let Some(deadline) = self.state.deadline.to_api() else { return; };
        for user in &self.users {
            user.do_send(WsResponse::MatchEvent(MatchEvent::UpdateDeadline(deadline)));
        }
    }

    fn broadcast_error(&self, error: WsNotifiedError) {
        for user in &self.users {
            user.do_send(WsResponse::NotifyError(error.clone()));
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
            ctx.notify(StartNewGame);
        }
    }

    fn player_win_game(&mut self, player: Player, reason: GameEndReason, ctx: &Context<Self>) {
        let user_idx = self.player_to_user_idx(player);
        tracing::info!(
            "User {} won the game in match {}. Reason: {reason:?}",
            self.users[user_idx].username(),
            self.info.id
        );
        let state = &mut self.state;
        let score = &mut state.player_states[user_idx].score;
        *score += 1;
        let number_of_games = self.info.settings.number_of_games;
        let match_is_end =
            *score > number_of_games / 2 || state.game_idx >= number_of_games as i32 - 1;
        state.phase = if match_is_end {
            MatchPhase::MatchEnded
        } else {
            MatchPhase::GameEnded
        };
        state.prev_end_state = Some(GameEndState {
            winner: player,
            reason,
        });
        if let Some(history) = self
            .history
            .as_mut()
            .crash_match_if_none("history is none before the match ends", ctx)
        {
            history.add_game(
                state.first_user_player,
                state.prev_actions.clone(),
                player,
                reason,
            );
        } else {
            return;
        }
        self.broadcast_game_end();

        if !match_is_end {
            ctx.notify_later(StartNewGame, BETWEEN_GAME_DELAY);
        } else {
            ctx.notify_later(EndMatch, BETWEEN_GAME_DELAY);
        }
        self.state.deadline.set_public(BETWEEN_GAME_DELAY);
        self.broadcast_deadline();
    }

    fn user_play(&mut self, player: Player, action: Action, ctx: &Context<Self>) -> Result<()> {
        self.state.user_play(player, action)?;
        self.state.deadline.unset();
        self.broadcast_last_action();

        if let Some(player) = self.state.game.winner() {
            self.player_win_game(player, GameEndReason::NoValidMove, ctx);
        } else {
            self.setup_next_deadline(ctx);
        }
        Ok(())
    }

    fn setup_next_deadline(&mut self, ctx: &Context<Self>) {
        let time_limit = if self.state.game.phase() == GamePhase::Pick {
            PICK_PHASE_TIME_LIMIT
        } else {
            self.info.settings.play_time_limit
        };
        let nonce = self.state.deadline.set_public(time_limit);
        let Some(player) = self
            .state
            .game
            .current_player()
            .crash_match_if_none("game not ended but no current player", ctx)
        else {
            return;
        };
        ctx.notify_later(PlayerTimeout { player, nonce }, time_limit + LEEWAY);
        self.broadcast_deadline();
    }

    fn set_users_idle(&mut self) {
        let user_states = User::lock_both_user_states(self.users.each_ref());
        for mut state in user_states {
            state.status = UserStatus::Idle;
        }
    }

    fn set_users_idle_and_update(&mut self) {
        self.set_users_idle();
        for user in &self.users {
            user.send_status_update();
        }
    }
}

impl Actor for MatchActor {
    fn started(&mut self, ctx: &Context<Self>) {
        let nonce = self.state.deadline.set();
        ctx.notify_later(CancelMatch { nonce }, MATCH_START_WAIT_TIME);
    }
}

pub struct SyncMatch {
    user: User,
}

impl Handler<SyncMatch> for MatchActor {
    type Output = Result<api::MatchState>;

    #[tracing::instrument(skip_all, fields(r#match = %self.info.id, action = "SyncMatch", user = ?msg.user.username()))]
    fn handle(&mut self, msg: SyncMatch, ctx: &Context<Self>) -> Self::Output {
        let user_idx = self.user_idx(msg.user.id()).ok_or(MatchError::NotInMatch)?;
        if !self.state.player_states[user_idx].is_ready {
            self.state.player_states[user_idx].is_ready = true;
            self.check_all_ready(ctx);
        }

        let match_state = match self.state.phase {
            MatchPhase::GameNotStarted => MatchInnerState::NotStarted,
            MatchPhase::GamePlaying => {
                let you = self
                    .user_player(msg.user.id())
                    .crash_match_if_none(
                        "can't find user in match after asserted the condition",
                        ctx,
                    )
                    .ok_or(MatchError::Unknown)?;
                MatchInnerState::Playing(api::GameState::GamePlaying(GameInnerState {
                    you,
                    prev_actions: self.state.prev_actions.clone(),
                }))
            }
            MatchPhase::GameEnded => {
                let you = self
                    .user_player(msg.user.id())
                    .crash_match_if_none(
                        "can't find user in match after asserting the condition",
                        ctx,
                    )
                    .ok_or(MatchError::Unknown)?;
                let info = self
                    .state
                    .prev_end_state
                    .crash_match_if_none("end state info is none", ctx)
                    .ok_or(MatchError::Unknown)?;

                MatchInnerState::Playing(api::GameState::GameEnded {
                    game_state: GameInnerState {
                        you,
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
            deadline: self.state.deadline.to_api(),
        }
        .into_perspective(user_idx))
    }
}

pub struct UserAction {
    pub user: User,
    pub action: MatchAction,
}

impl Handler<UserAction> for MatchActor {
    type Output = Result<()>;

    #[tracing::instrument(skip_all, fields(r#match = %self.info.id, action = "UserAction", user = ?msg.user.username()))]
    fn handle(&mut self, msg: UserAction, ctx: &Context<Self>) -> Self::Output {
        let player = self
            .user_player(msg.user.id())
            .ok_or(MatchError::NotInMatch)?;
        match msg.action {
            MatchAction::Play(action) => self.user_play(player, action, ctx),
        }
    }
}

struct StartNewGame;

impl Handler<StartNewGame> for MatchActor {
    type Output = ();

    #[tracing::instrument(skip_all, fields(r#match = %self.info.id, message = "StartNewGame"))]
    fn handle(&mut self, _msg: StartNewGame, ctx: &Context<Self>) -> Self::Output {
        self.state.deadline.unset();

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

        self.broadcast_new_game();
        self.setup_next_deadline(ctx)
    }
}

struct EndMatch;

impl Handler<EndMatch> for MatchActor {
    type Output = ();

    #[tracing::instrument(skip_all, fields(r#match = %self.info.id, message = "EndMatch"))]
    fn handle(&mut self, _msg: EndMatch, ctx: &Context<Self>) -> Self::Output {
        let Some(history) = self
            .history
            .take()
            .crash_match_if_none("history is empty before match ends", ctx)
        else { return; };
        let end_time = Utc::now();
        spawn(async move {
            if let Err(err) = history.save(end_time).await {
                tracing::error!("failed to save history: {}", err);
            }
        });
        tracing::info!(
            "Match {} ended: {} ({}) : {} ({})",
            self.info.id,
            self.users[0].username(),
            self.state.player_states[0].score,
            self.users[1].username(),
            self.state.player_states[1].score
        );
        self.broadcast_match_end();
        self.set_users_idle();
    }
}

struct CancelMatch {
    nonce: DeadlineNonce,
}

impl Handler<CancelMatch> for MatchActor {
    type Output = ();

    #[tracing::instrument(skip_all, fields(r#match = %self.info.id, message = "CancelGame"))]
    fn handle(&mut self, msg: CancelMatch, _ctx: &Context<Self>) -> Self::Output {
        if !self.state.deadline.expiration_is_valid(msg.nonce) {
            return;
        }
        self.broadcast_error(WsNotifiedError::GameCanceled);
        self.set_users_idle_and_update();
    }
}

struct CrashMatch;

impl Handler<CrashMatch> for MatchActor {
    type Output = ();

    #[tracing::instrument(skip_all, fields(r#match = %self.info.id, message = "CrashGame"))]
    fn handle(&mut self, _msg: CrashMatch, ctx: &Context<Self>) -> Self::Output {
        self.broadcast_error(WsNotifiedError::GameCrashed);
        self.set_users_idle_and_update();
        ctx.stop();
    }
}

struct PlayerTimeout {
    player: Player,
    nonce: DeadlineNonce,
}

impl Handler<PlayerTimeout> for MatchActor {
    type Output = ();

    fn handle(&mut self, msg: PlayerTimeout, ctx: &Context<Self>) -> Self::Output {
        if !self.state.deadline.expiration_is_valid(msg.nonce) {
            return;
        }
        let game = &self.state.game;
        if game.current_player() != Some(msg.player) {
            return;
        }
        match game.phase() {
            GamePhase::Pick => {
                let hexo = if let Some(hexo) = game
                    .inventory()
                    .remaining_hexos()
                    .iter()
                    .next()
                    .crash_match_if_none("there is no remaining hexos in pick phase", ctx)
                {
                    hexo
                } else {
                    return;
                };
                let _ = self.user_play(msg.player, Action::Pick(hexo), ctx);
            }
            GamePhase::Place => {
                self.player_win_game(msg.player.other(), GameEndReason::TimeLimitExceed, ctx);
            }
            _ => (),
        }
    }
}

pub fn to_match_settings(config: MatchConfig) -> MatchSettings {
    let (number_of_games, play_time_limit) = match config {
        MatchConfig::Normal => (1, Duration::from_secs(40)),
        MatchConfig::KnockoutStage => (3, Duration::from_secs(30)),
        MatchConfig::ChampionshipStage => (2, Duration::from_secs(30)),
    };
    MatchSettings {
        config,
        number_of_games,
        play_time_limit,
    }
}

impl MatchInfo {
    fn new(users: &[User; 2], config: MatchConfig, match_token: Option<MatchToken>) -> Self {
        Self {
            id: MatchId(Uuid::new_v4()),
            settings: to_match_settings(config),
            match_token,
            user_data: users.each_ref().map(|u| u.to_api()),
        }
    }

    fn to_api(&self) -> api::MatchInfo {
        api::MatchInfo {
            id: self.id,
            num_games: self.settings.number_of_games,
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
            deadline: Deadline::new(),
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
        match scores[0].cmp(&scores[1]) {
            Ordering::Greater => Some(0),
            Ordering::Less => Some(0),
            Ordering::Equal => None,
        }
    }

    fn winner_from_user(&self, idx: usize) -> MatchWinner {
        match self.winner() {
            None => MatchWinner::Tie,
            Some(jdx) if jdx == idx => MatchWinner::You,
            _ => MatchWinner::They,
        }
    }

    #[allow(dead_code)]
    fn fast_forward_to_place(&mut self) {
        for hexo in hexomino_core::Hexo::all_hexos() {
            let _ = self.user_play(self.game.current_player().unwrap(), Action::Pick(hexo));
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

#[derive(PartialEq, Eq, Clone, Copy)]
struct DeadlineNonce(u64);

impl DeadlineNonce {
    fn advance(&mut self) {
        self.0 += 1;
    }
}

struct Deadline {
    nonce: DeadlineNonce,
    inner: Option<DeadlineInner>,
}

struct DeadlineInner {
    time: DateTime<Utc>,
    duration: Duration,
}

impl Deadline {
    fn new() -> Self {
        Self {
            nonce: DeadlineNonce(0),
            inner: None,
        }
    }

    fn set(&mut self) -> DeadlineNonce {
        self.nonce.advance();
        self.nonce
    }

    fn set_public(&mut self, after: Duration) -> DeadlineNonce {
        self.set();
        self.inner = Some(DeadlineInner {
            time: Utc::now() + chrono::Duration::from_std(after).expect("duration out of bound"),
            duration: after,
        });
        self.nonce
    }

    fn unset(&mut self) {
        self.nonce.advance();
        self.inner = None;
    }

    fn expiration_is_valid(&self, nonce: DeadlineNonce) -> bool {
        self.inner.is_some() && self.nonce == nonce
    }

    fn to_api(&self) -> Option<api::Deadline> {
        self.inner.as_ref().map(|inner| api::Deadline {
            time: inner.time,
            duration: inner.duration,
        })
    }
}

trait IntoPerspective: Sized {
    fn flip(self) -> Self;
    fn into_perspective(self, user_idx: usize) -> Self {
        match user_idx {
            0 => self,
            1 => self.flip(),
            _ => {
                panic!("invalid user idx {user_idx}");
            }
        }
    }
}

impl IntoPerspective for api::MatchState {
    fn flip(mut self) -> Self {
        self.info = self.info.flip();
        self.scores.swap(0, 1);
        self
    }
}

impl IntoPerspective for api::MatchInfo {
    fn flip(mut self) -> Self {
        self.user_data.swap(0, 1);
        self
    }
}

impl IntoPerspective for api::GameEndInfo {
    fn flip(mut self) -> Self {
        self.scores.swap(0, 1);
        self
    }
}

impl IntoPerspective for api::MatchEndInfo {
    fn flip(mut self) -> Self {
        self.scores.swap(0, 1);
        self
    }
}

trait OptionExt {
    fn crash_match_if_none(self, msg: &str, ctx: &Context<MatchActor>) -> Self;
}

impl<T> OptionExt for Option<T> {
    fn crash_match_if_none(self, msg: &str, ctx: &Context<MatchActor>) -> Self {
        match self {
            Some(_) => self,
            None => {
                tracing::error!("{}", msg);
                ctx.notify(CrashMatch);
                self
            }
        }
    }
}

trait ResultExt {
    fn crash_match_if_err(self, ctx: &Context<MatchActor>) -> Self;
}

impl<T> ResultExt for Result<T> {
    fn crash_match_if_err(self, ctx: &Context<MatchActor>) -> Self {
        match self {
            Ok(_) => self,
            Err(ref err) => {
                tracing::error!("{}", err);
                ctx.notify(CrashMatch);
                self
            }
        }
    }
}
