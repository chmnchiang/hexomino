use hexomino_core::{Action, Player};
use uuid::Uuid;

use crate::{derive_api_data, Api, User};

derive_api_data! {
    pub struct GameInfo {
        pub game_id: GameId,
        pub users: [User; 2],
        pub me: Player,
    }
    #[derive(Copy, PartialEq, Eq, Hash)]
    #[derive(derive_more::Display, derive_more::FromStr)]
    pub struct GameId(pub Uuid);
    pub enum GameAction {
        Connected,
        Play(Action),
    }
    pub enum GameEvent {
        UserPlay(Action),
        GameEnd(GameEndInfo),
    }
    pub struct GameEndInfo {
        reason: GameEndReason
    }
    pub enum GameEndReason {
        NoValidMove,
    }
    #[derive(thiserror::Error)]
    pub enum GameError {
        #[error("user is not in the game")]
        NotInGame,
        #[error("it is not your turn")]
        NotYourTurn,
        #[error("cannot perform game action: {0}")]
        GameActionError(String),
    }
}

derive_api_data! {
    pub struct GameActionApi;
}

pub type GameActionRequest = GameAction;
pub type GameActionResponse = Result<(), GameError>;
impl Api for GameActionApi {
    type Request = GameActionRequest;
    type Response = GameActionResponse;
}
