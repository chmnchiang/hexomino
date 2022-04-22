use hexomino_core::{Action, Player};
use serde::{Deserialize, Serialize};

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

pub enum Request {
    GetRooms,
    CreateRoom,
    JoinRoom(RoomId),
}

pub enum Response {
    RoomMsg(RoomMsg),
    GameMsg(GameMsg),
    Error(Error),
}

pub struct User {
    pub id: UserId,
    pub name: String,
}

#[derive(Copy, PartialEq, Eq)]
pub struct UserId(pub i64);


pub enum RoomMsg {
    SyncRooms(Vec<Room>),
    JoinRoom(Room),
}

pub struct Room {
    pub id: RoomId,
    pub users: Vec<User>,
}

#[derive(Hash, Copy, PartialEq, Eq)]
pub struct RoomId(pub i64);

pub enum GameMsg {
    UserPlay(UserPlay),
}

pub struct UserPlay {
    pub player: Player,
    pub action: Action,
}

}

pub type WsConnectResult = Result<HelloFromKernel>;

//pub struct WsConnect
