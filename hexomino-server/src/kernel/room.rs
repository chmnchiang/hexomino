use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};

use api::{RoomAction, RoomError, RoomId, UserId, WsResponse, GameInfo};
use dashmap::DashMap;
use itertools::Itertools;
use parking_lot::Mutex;

use crate::{
    kernel::{user::UserStatus, User},
    result::ApiResult,
};

use super::game::Game;

type Result<T> = ApiResult<T, RoomError>;

pub struct RoomManager {
    rooms: DashMap<RoomId, Arc<Room>>,
    counter: AtomicI64,
}

pub struct Room {
    id: RoomId,
    state: Mutex<RoomState>,
}

pub struct RoomState {
    users: Vec<RoomUser>,
}

impl RoomState {
    fn get_user(&mut self, user_id: UserId) -> Option<&mut RoomUser> {
        for user in &mut self.users {
            if user.user.id() == user_id {
                return Some(user);
            }
        }
        None
    }

    fn all_ready(&self) -> bool {
        self.users.len() == 2 && self.users[0].is_ready && self.users[1].is_ready
    }

    fn start_game(&self) {
        let u1 = self.users[0].user.clone();
        let u2 = self.users[1].user.clone();
        let game = Game::new(u1.clone(), u2.clone());
        let mut u1_state = u1.state().write();
        let mut u2_state = u2.state().write();
        u1_state.status = UserStatus::InGame(game.clone());
        u2_state.status = UserStatus::InGame(game.clone());

        u1.spawn_send(WsResponse::GameStart(GameInfo {
            game_id: game.id(),
            users: [u1.to_api(), u2.to_api()],
            me: hexomino_core::Player::First,
        }));
        u2.spawn_send(WsResponse::GameStart(GameInfo {
            game_id: game.id(),
            users: [u1.to_api(), u2.to_api()],
            me: hexomino_core::Player::Second,
        }));
    }
}

impl Room {
    fn new(id: RoomId) -> Self {
        Self {
            id,
            state: Mutex::new(RoomState { users: vec![] }),
        }
    }

    fn to_api(&self) -> api::Room {
        api::Room {
            id: self.id,
            users: self
                .state
                .lock()
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
                .state
                .lock()
                .users
                .iter()
                .map(|user| user.to_api())
                .collect_vec(),
        }
    }

    fn user_enter(&self, user: User) -> Result<()> {
        let mut state = self.state.lock();
        if state.users.len() >= 2 {
            Err(RoomError::RoomIsFull(self.id))?
        }
        state.users.push(RoomUser {
            user,
            is_ready: false,
        });
        Ok(())
    }

    fn broadcast_update(&self) {
        let room = self.to_joined_room();
        for user in self.state.lock().users.iter().map(|u| u.user.clone()) {
            user.spawn_send(WsResponse::RoomUpdate(room.clone()));
        }
    }

    pub fn room_action(&self, user_id: UserId, action: RoomAction) -> Result<()> {
        {
            let mut state = self.state.lock();
            let user = state.get_user(user_id).ok_or(RoomError::NotInRoom)?;

            match action {
                RoomAction::Ready => {
                    user.is_ready = true;
                    if state.all_ready() {
                        state.start_game();
                        return Ok(())
                    }
                }
                RoomAction::Unready => {
                    user.is_ready = false;
                }
            }
        }
        self.broadcast_update();
        Ok(())
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

    pub fn get(&self, room_id: RoomId) -> Result<Arc<Room>> {
        Ok(self
            .rooms
            .get(&room_id)
            .map(|r| r.value().clone())
            .ok_or(RoomError::RoomNotFound(room_id))?)
    }

    pub fn list_rooms(&self) -> Vec<api::Room> {
        self.rooms.iter().map(|room| room.to_api()).collect_vec()
    }

    pub fn create_room(&self, user: User) -> Result<RoomId> {
        let user_clone = user.clone();
        let mut user_state = user.state().write();
        let UserStatus::Idle = user_state.status else { return Err(RoomError::UserBusy)? };

        let room_id = RoomId(self.counter.fetch_add(1, Ordering::Relaxed));
        let room = Room::new(room_id);
        room.user_enter(user_clone)?;
        self.rooms.insert(room_id, Arc::new(room));
        user_state.status = UserStatus::InRoom(room_id);
        user.spawn_send(WsResponse::MoveToRoom(room_id));
        Ok(room_id)
    }

    pub fn join_room(&self, user: User, room_id: RoomId) -> Result<()> {
        let user_clone = user.clone();
        let mut user_state = user.state().write();
        let UserStatus::Idle = user_state.status else { return Err(RoomError::UserBusy)? };

        let room = self.get(room_id)?;
        room.user_enter(user_clone)?;
        user_state.status = UserStatus::InRoom(room_id);
        user.spawn_send(WsResponse::MoveToRoom(room_id));
        room.broadcast_update();
        Ok(())
    }

    pub fn get_joined_room(&self, room_id: RoomId) -> Result<api::JoinedRoom> {
        Ok(self.get(room_id)?.to_joined_room())
    }
}
