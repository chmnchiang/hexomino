use crate::{derive_api_data, Api, JoinedRoom, MatchEvent};

pub struct StartWsApi;
impl Api for StartWsApi {
    type Request = StartWsRequest;
    type Response = Result<StartWsResponse, StartWsError>;
}

derive_api_data! {
    pub struct StartWsRequest {
        pub token: String,
    }
    pub struct StartWsResponse {
        pub username: String,
    }
    #[derive(thiserror::Error)]
    pub enum StartWsError {
        #[error("timeout when receiving initial websocket message handshake")]
        Timeout,
        #[error("fail to establish initial websocket message handshake")]
        InitialHandshakeFailed,
        #[error("fail to authenticate websocket stream")]
        WsAuthError,
        #[error("internal error")]
        InternalError,
    }
}

derive_api_data! {

pub enum WsRequest {}
pub enum WsResponse {
    UserStatusUpdate(UserStatus),
    RoomUpdate(JoinedRoom),
    MatchEvent(MatchEvent),
    NotifyError(WsNotifiedError),
}

pub enum UserStatus {
    Idle,
    InRoom,
    InGame,
}

#[derive(thiserror::Error)]
pub enum WsNotifiedError {
    #[error("The game is canceled because one of the player disconnected before the game started.")]
    GameCanceled,
    #[error("The game crashed due to internal error. Please notify the admin and restart another game.")]
    GameCrashed,
}

}

pub type WsResult = WsResponse;
