use std::time::Duration;

use thiserror::Error;

use crate::{derive_api_data, Api, User};

derive_api_data! {
    pub struct Room {
        pub id: RoomId,
        pub match_token: Option<MatchToken>,
        pub users: Vec<User>,
    }
    #[derive(Hash, Copy, PartialEq, Eq, PartialOrd, Ord)]
    #[derive(derive_more::Display, derive_more::FromStr)]
    pub struct RoomId(pub i64);

    pub struct RoomUser {
        pub user: User,
        pub is_ready: bool,
    }

    pub struct JoinedRoom {
        pub id: RoomId,
        pub match_token: Option<MatchToken>,
        pub users: Vec<RoomUser>,
        pub settings: MatchSettings,
    }

    pub enum RoomAction {
        Ready,
        UndoReady,
        SetConfig(MatchConfig),
    }

    pub struct MatchSettings {
        pub config: MatchConfig,
        pub number_of_games: u32,
        pub play_time_limit: Duration,
    }

    #[derive(Copy, strum::Display, strum::EnumString, strum::IntoStaticStr)]
    pub enum MatchConfig {
        Normal,
        KnockoutStage,
        ChampionshipStage,
    }

    #[derive(Hash, PartialEq, Eq, derive_more::Display)]
    pub struct MatchToken(pub String);
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
        #[error("match token is not valid.")]
        MatchTokenNotValid,
    }
}

type Result<T> = std::result::Result<T, RoomError>;

derive_api_data! {
    pub struct ListRoomsApi;
    pub struct JoinRoomApi;
    pub struct LeaveRoomApi;
    pub struct CreateRoomApi;
    pub struct CreateOrJoinMatchRoomApi;
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

pub type LeaveRoomRequest = ();
pub type LeaveRoomResponse = ();
impl Api for LeaveRoomApi {
    type Request = LeaveRoomRequest;
    type Response = Result<LeaveRoomResponse>;
}

pub type CreateRoomRequest = ();
pub type CreateRoomResponse = RoomId;
impl Api for CreateRoomApi {
    type Request = CreateRoomRequest;
    type Response = Result<CreateRoomResponse>;
}

pub type CreateOrJoinMatchRoomRequest = MatchToken;
pub type CreateOrJoinMatchRoomResponse = RoomId;
impl Api for CreateOrJoinMatchRoomApi {
    type Request = CreateOrJoinMatchRoomRequest;
    type Response = Result<CreateRoomResponse>;
}

pub type GetRoomRequest = ();
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
