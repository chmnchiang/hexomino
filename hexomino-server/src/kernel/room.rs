use std::sync::atomic::{AtomicI64, Ordering};

use api::{RoomError, RoomId};
use dashmap::DashMap;
use guard::guard;
use itertools::Itertools;


use crate::{kernel::{user::UserStatus, User}, result::ApiResult};

type Result<T> = ApiResult<T, RoomError>;

pub struct RoomManager {
    rooms: DashMap<RoomId, Room>,
    counter: AtomicI64,
}

struct Room {
    pub id: RoomId,
    pub users: Vec<User>,
}

impl Room {
    fn new(id: RoomId) -> Self {
        Self { id, users: vec![] }
    }
}

impl From<&Room> for api::Room {
    fn from(room: &Room) -> Self {
        Self {
            id: room.id,
            users: room.users.iter().map(From::from).collect_vec(),
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

    pub fn get(&self, room_id: RoomId) -> Result<api::Room> {
        let room = self.rooms
            .get(&room_id)
            .ok_or_else(|| RoomError::RoomNotFound(room_id))?;
        Ok(room.value().into())
    }

    pub fn list_rooms(&self) -> Vec<api::Room> {
        self.rooms
            .iter()
            .map(|r| From::from(r.value()))
            .collect_vec()
    }

    pub fn create_room(&self, user: User) -> Result<RoomId> {
        println!("create");
        let user_clone = user.clone();
        let user_state = user.state().write();
        guard!(let UserStatus::Idle = user_state.status
            else { return Err(RoomError::UserBusy)? });

        let id = RoomId(self.counter.fetch_add(1, Ordering::Relaxed));
        let mut room = Room::new(id);
        room.users.push(user_clone);
        self.rooms.insert(id, room);
        Ok(id)
    }

    pub fn join_room(&self, user: User, room_id: RoomId) -> Result<()> {
        let mut room = self
            .rooms
            .get_mut(&room_id)
            .ok_or_else(|| RoomError::RoomNotFound(room_id))?;
        let room = room.value_mut();
        if room.users.len() >= 2 {
            return Err(RoomError::RoomIsFull(room_id))?;
        }
        room.users.push(user);
        Ok(())
    }
}
