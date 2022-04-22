use std::sync::atomic::{AtomicI64, Ordering};

use crate::kernel::User;
use api::{cerr, RoomId};
use dashmap::DashMap;
use itertools::Itertools;
use parking_lot::RwLock;

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

    pub fn get_rooms(&self) -> Vec<api::Room> {
        self.rooms
            .iter()
            .map(|r| From::from(r.value()))
            .collect_vec()
    }

    pub fn create_room(&self, user: User) -> api::Result<api::Room> {
        let id = RoomId(self.counter.fetch_add(1, Ordering::Relaxed));
        let mut room = Room::new(id);
        room.users.push(user);
        let result = Ok((&room).into());
        self.rooms.insert(id, room);
        result
    }

    pub fn join_room(&self, user: User, room_id: RoomId) -> api::Result<api::Room> {
        let mut room = self
            .rooms
            .get_mut(&room_id)
            .ok_or_else(|| cerr!("room id: {} not found", room_id.0))?;
        let room = room.value_mut();
        if room.users.len() >= 2 {
            Err(cerr!("room id: {} is full", room.id.0))?;
        }
        room.users.push(user);
        Ok((&*room).into())
    }
}
