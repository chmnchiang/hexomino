use std::sync::atomic::{AtomicI64, Ordering};

use api::{RoomError, RoomId, WsResponse};
use dashmap::DashMap;
use itertools::Itertools;

use crate::{
    kernel::{user::UserStatus, User},
    result::ApiResult,
};

type Result<T> = ApiResult<T, RoomError>;

pub struct RoomManager {
    rooms: DashMap<RoomId, Room>,
    counter: AtomicI64,
}

struct Room {
    pub id: RoomId,
    pub users: Vec<RoomUser>,
}

impl Room {
    fn new(id: RoomId) -> Self {
        Self { id, users: vec![] }
    }

    fn to_api(&self) -> api::Room {
        api::Room {
            id: self.id,
            users: self
                .users
                .iter()
                .map(|user| user.user.to_api())
                .collect_vec(),
        }
    }

    fn to_joined_room(&self) -> api::JoinedRoom {
        api::JoinedRoom {
            id: self.id,
            users: self
                .users
                .iter()
                .map(|user| user.to_api())
                .collect_vec(),
        }
    }


    fn user_enter(&mut self, user: User) -> Result<()>  {
        if self.users.len() >= 2 {
            Err(RoomError::RoomIsFull(self.id))?
        }
        self.users.push(RoomUser {
            user,
            is_ready: false,
        });
        Ok(())
    }

    fn broadcast_update(&self) {
        let room = self.to_joined_room();
        for user in self.users.iter().map(|u| u.user.clone()) {
            user.spawn_send(WsResponse::RoomUpdate(room.clone()));
        }
    }
}

struct RoomUser {
    user: User,
    is_ready: bool,
}

impl RoomUser {
    fn to_api(&self) -> api::RoomUser {
        api::RoomUser {
            user: self.user.to_api(),
            is_ready: self.is_ready,
        }
    }
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: DashMap::new(),
            counter: AtomicI64::new(0),
        }
    }

    pub fn get(&self, room_id: RoomId) -> Result<api::JoinedRoom> {
        let room = self
            .rooms
            .get(&room_id)
            .ok_or(RoomError::RoomNotFound(room_id))?;
        Ok(room.value().to_joined_room())
    }

    pub fn list_rooms(&self) -> Vec<api::Room> {
        self.rooms.iter().map(|room| room.to_api()).collect_vec()
    }

    pub fn create_room(&self, user: User) -> Result<RoomId> {
        let user_clone = user.clone();
        let mut user_state = user.state().write();
        let UserStatus::Idle = user_state.status else { return Err(RoomError::UserBusy)? };

        let room_id = RoomId(self.counter.fetch_add(1, Ordering::Relaxed));
        let mut room = Room::new(room_id);
        room.user_enter(user_clone)?;
        self.rooms.insert(room_id, room);
        user_state.status = UserStatus::InRoom(room_id);
        user.spawn_send(WsResponse::MoveToRoom(room_id));
        Ok(room_id)
    }

    pub fn join_room(&self, user: User, room_id: RoomId) -> Result<()> {
        let user_clone = user.clone();
        let mut user_state = user.state().write();
        let UserStatus::Idle = user_state.status else { return Err(RoomError::UserBusy)? };

        let mut room = self
            .rooms
            .get_mut(&room_id)
            .ok_or(RoomError::RoomNotFound(room_id))?;
        let room = room.value_mut();
        room.user_enter(user_clone)?;
        user_state.status = UserStatus::InRoom(room_id);
        user.spawn_send(WsResponse::MoveToRoom(room_id));
        room.broadcast_update();
        Ok(())
    }
}
