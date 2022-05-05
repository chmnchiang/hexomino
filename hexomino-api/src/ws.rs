use crate::{derive_api_data, Api, RoomId, JoinedRoom};

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
    MoveToRoom(RoomId),
    RoomUpdate(JoinedRoom),
    GameStart(GameInfo),
    GameEvent(GameEvent),
    GameStateSync(GameState),
}

#[derive(thiserror::Error)]
pub enum WsError {}

}

pub type WsResult = WsResponse;
