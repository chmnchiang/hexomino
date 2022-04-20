use serde::{Deserialize, Serialize};
use hexomino_core::{Player, Action};

mod error;

macro_rules! derive_api {
    () => {};
    ($item:item $($rest:item)*) => {
        #[derive(Serialize, Deserialize, Debug, Clone)]
        $item

        derive_api!($($rest)*);
    };
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Error {
    Client(String),
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

derive_api! {

pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

pub struct AuthResponse {
    pub username: String,
    pub token: String,
}

pub struct StartWsConnection {
    pub token: String,
}

pub struct HelloFromKernel {
    pub username: String,
}

pub enum MsgFromServer {
    RoomMsg(RoomMsg),
    GameMsg(GameMsg),
}

pub enum RoomMsg {
    SyncRooms(SyncRooms),
}

pub struct SyncRooms {
    rooms: Vec<Room>
}

pub struct Room;

pub enum GameMsg {
    UserPlay(UserPlay),
}

pub struct UserPlay {
    player: Player,
    action: Action,
}

}

pub type WsConnectResult = Result<HelloFromKernel>;

//pub struct WsConnect
