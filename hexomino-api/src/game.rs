use hexomino_core::{Action, Player};
use uuid::Uuid;

use crate::{derive_api_data, Api, User};

derive_api_data! {
    pub struct MatchState {
        pub info: MatchInfo,
        pub game_idx: i32,
        pub scores: [u32; 2],
        pub state: MatchInnerState,
    }
    pub struct MatchInfo {
        pub id: MatchId,
        pub num_games: u32,
        pub user_data: [User; 2],
    }
    #[derive(Copy, PartialEq, Eq, Hash)]
    #[derive(derive_more::Display, derive_more::FromStr)]
    pub struct MatchId(pub Uuid);

    pub enum MatchInnerState {
        NotStarted,
        Playing(GameState),
        Ended { winner: MatchWinner },
    }

    pub enum GameState {
        GamePlaying(GameInnerState),
        GameEnded {
            game_state: GameInnerState,
            end_state: GameEndState,
        },
    }
    pub struct GameEndState {
        pub winner: Player,
        pub reason: GameEndReason,
    }
    pub struct GameInnerState {
        pub you: Player,
        pub prev_actions: Vec<Action>,
    }
    pub enum MatchAction {
        Play(Action),
    }
    pub enum MatchEvent {
        GameStart { you: Player },
        UserPlay(UserPlay),
        GameEnd(GameEndInfo),
        MatchEnd(MatchEndInfo),
    }
    pub struct UserPlay {
        pub action: Action,
        pub idx: u32,
    }
    pub struct GameEndInfo {
        pub end_state: GameEndState,
        pub scores: [u32; 2],
    }
    pub struct MatchEndInfo {
        pub scores: [u32; 2],
        pub winner: MatchWinner,
    }
    #[derive(Copy)]
    pub enum MatchWinner {
        You,
        They,
        Tie,
    }
    #[derive(Copy)]
    pub enum GameEndReason {
        NoValidMove,
    }
    #[derive(thiserror::Error)]
    pub enum MatchError {
        #[error("user is not in the match")]
        NotInMatch,
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
