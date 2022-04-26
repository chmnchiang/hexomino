use thiserror::Error;

use crate::{derive_api_data, User, Api};

derive_api_data! {
    pub struct Room {
        pub id: RoomId,
        pub users: Vec<User>,
    }
    #[derive(Hash, Copy, PartialEq, Eq, derive_more::Display)]
    #[display(fmt = "{}",  _0)]
    pub struct RoomId(pub i64);
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
    }
}
type Result<T> = std::result::Result<T, RoomError>;

derive_api_data! {
    pub struct ListRoomsApi;
}
pub type ListRoomsRequest = ();
pub type ListRoomsResponse = Vec<Room>;
impl Api for ListRoomsApi {
    type Request = ListRoomsRequest;
    type Response = ListRoomsResponse;
}

derive_api_data! {
    pub struct JoinRoomApi;
}
pub type JoinRoomRequest = RoomId;
pub type JoinRoomResponse = ();
impl Api for JoinRoomApi {
    type Request = JoinRoomRequest;
    type Response = Result<JoinRoomResponse>;
}

derive_api_data! {
    pub struct CreateRoomApi;
}
pub type CreateRoomRequest = ();
pub type CreateRoomResponse = RoomId;
impl Api for CreateRoomApi {
    type Request = CreateRoomRequest;
    type Response = Result<CreateRoomResponse>;
}

derive_api_data! {
    pub struct GetRoomApi;
}
pub type GetRoomRequest = RoomId;
pub type GetRoomResponse = Room;
impl Api for GetRoomApi {
    type Request = GetRoomRequest;
    type Response = Result<GetRoomResponse>;
}
