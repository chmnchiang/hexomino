use crate::derive_api_data;

derive_api_data! {
    pub struct GameInfo;
    pub enum GameEvent {
        UserPlay(UserPlay),
        GameEnd(GameEndInfo),
    }
    pub struct GameEvent {
    }
    #[derive(thiserror::Error)]
    pub enum GameError {
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
