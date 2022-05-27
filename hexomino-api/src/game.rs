use hexomino_core::{Action, Player};
use uuid::Uuid;

use crate::{derive_api_data, Api, User};

derive_api_data! {
    pub struct MatchState {
        pub info: MatchInfo,
        pub game_idx: i32,
        pub game_status: GameStatus,
        pub scores: [u32; 2],
        pub you: Player,
        pub prev_actions: Vec<Action>,
    }
    pub struct MatchInfo {
        pub id: MatchId,
        pub num_games: u32,
        pub user_data: [User; 2],
    }
    #[derive(Copy, PartialEq, Eq, Hash)]
    #[derive(derive_more::Display, derive_more::FromStr)]
    pub struct MatchId(pub Uuid);

    pub enum GameStatus {
        NotStarted,
        Playing,
        Ended(GameEndReason),
    }

    pub enum MatchAction {
        Play(Action),
    }
    pub enum GameEvent {
        GameStart(GameStartInfo),
        UserPlay(UserPlay),
        GameEnd(GameEndInfo),
    }
    pub struct GameStartInfo {
        pub you: Player,
    }
    pub struct UserPlay {
        pub action: Action,
        pub idx: u32,
    }
    pub struct GameEndInfo {
        pub reason: GameEndReason,
        pub scores: [u32; 2],
        pub match_is_end: bool,
    }
    #[derive(Copy)]
    pub enum GameEndReason {
        NoValidMove,
    }
    #[derive(thiserror::Error)]
    pub enum MatchError {
        #[error("user is not in the match")]
        NotInMatch,
        #[error("it is not your turn")]
        NotYourTurn,
        #[error("cannot perform game action: {0}")]
        GameActionError(String),
    }
}

derive_api_data! {
    pub struct SyncMatchApi;
    pub struct MatchActionApi;
}

pub type SyncMatchRequest = ();
pub type SyncMatchResponse = Result<MatchState, MatchError>;
impl Api for SyncMatchApi {
    type Request = SyncMatchRequest;
    type Response = SyncMatchResponse;
}

pub type MatchActionRequest = MatchAction;
pub type MatchActionResponse = Result<(), MatchError>;
impl Api for MatchActionApi {
    type Request = MatchActionRequest;
    type Response = MatchActionResponse;
}
