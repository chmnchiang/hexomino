use thiserror::Error;

use crate::{derive_api_data, Api, User, UserId};

derive_api_data! {
    pub struct Room {
        pub id: RoomId,
        pub users: Vec<User>,
    }
    #[derive(Hash, Copy, PartialEq, Eq)]
    #[derive(derive_more::Display, derive_more::FromStr)]
    pub struct RoomId(pub i64);

    pub struct RoomUser {
        pub user: User,
        pub is_ready: bool,
    }

    pub struct JoinedRoom {
        pub id: RoomId,
        pub users: Vec<RoomUser>,
    }

    pub enum RoomAction {
        Ready,
        Unready,
    }
}

derive_api_data! {
    #[derive(Error)]
    pub enum RoomError {
        #[error("user is already playing or in another room")]
        UserBusy,
        #[error("cannot find room id={0}")]
        RoomNotFound(RoomId),
        #[error("room id={0} is full")]
        RoomIsFull(RoomId),
        #[error("user is not in the room")]
        NotInRoom,
    }
}

type Result<T> = std::result::Result<T, RoomError>;

derive_api_data! {
    pub struct ListRoomsApi;
    pub struct JoinRoomApi;
    pub struct CreateRoomApi;
    pub struct GetRoomApi;
    pub struct RoomActionApi;
}
pub type ListRoomsRequest = ();
pub type ListRoomsResponse = Vec<Room>;
impl Api for ListRoomsApi {
    type Request = ListRoomsRequest;
    type Response = ListRoomsResponse;
}

pub type JoinRoomRequest = RoomId;
pub type JoinRoomResponse = ();
impl Api for JoinRoomApi {
    type Request = JoinRoomRequest;
    type Response = Result<JoinRoomResponse>;
}

pub type CreateRoomRequest = ();
pub type CreateRoomResponse = RoomId;
impl Api for CreateRoomApi {
    type Request = CreateRoomRequest;
    type Response = Result<CreateRoomResponse>;
}

pub type GetRoomRequest = RoomId;
pub type GetRoomResponse = JoinedRoom;
impl Api for GetRoomApi {
    type Request = GetRoomRequest;
    type Response = Result<GetRoomResponse>;
}

pub type RoomActionRequest = RoomAction;
pub type RoomActionResponse = ();
impl Api for RoomActionApi {
    type Request = RoomActionRequest;
    type Response = Result<()>;
}
