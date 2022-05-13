use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};

use api::{GameInfo, RoomAction, RoomError, RoomId, UserId, WsResponse};
use itertools::Itertools;
use parking_lot::{RwLock, RwLockWriteGuard};

use crate::{
    kernel::{user::UserStatus, User},
    result::ApiResult,
};

use super::{
    actor::{Actor, Addr, Context, Handler},
    game::GameActor,
    user::UserState,
};

type Result<T> = ApiResult<T, RoomError>;

pub struct RoomManagerHandle {
    cached_rooms: Arc<RwLock<Vec<api::Room>>>,
    addr: Addr<RoomManager>,
}

impl RoomManagerHandle {
    pub fn new() -> Self {
        let room_manager = RoomManager::new();
        let cached_rooms = room_manager.cached_rooms.clone();
        Self {
            cached_rooms,
            addr: room_manager.start(),
        }
    }

    pub fn list_rooms(&self) -> Vec<api::Room> {
        self.cached_rooms.read().clone()
    }
    pub async fn create_room(&self, user: User) -> Result<RoomId> {
        self.addr.send(CreateRoom { user }).await
    }
    pub async fn join_room(&self, user: User, room_id: RoomId) -> Result<()> {
        self.addr.send(JoinRoom { user, room_id }).await
    }
    pub async fn get_joined_room(&self, user: User, room_id: RoomId) -> Result<api::JoinedRoom> {
        self.addr
            .send(GetJoinedRoom {
                user_id: user.id(),
                room_id,
            })
            .await
    }
    pub async fn user_room_action(
        &self,
        user: User,
        room_id: RoomId,
        action: RoomAction,
    ) -> Result<()> {
        self.addr
            .send(UserRoomAction {
                user_id: user.id(),
                room_id,
                action,
            })
            .await
    }
}

struct RoomManager {
    rooms: HashMap<RoomId, Room>,
    counter: AtomicI64,
    cached_rooms: Arc<RwLock<Vec<api::Room>>>,
}

pub struct Room {
    id: RoomId,
    users: Vec<RoomUser>,
}

impl Actor for RoomManager {}

struct CreateRoom {
    user: User,
}

impl Handler<CreateRoom> for RoomManager {
    type Output = Result<RoomId>;
    fn handle(&mut self, msg: CreateRoom, _ctx: &Context<Self>) -> Self::Output {
        let user = msg.user;
        let user_clone = user.clone();
        let mut user_state = user.state().write();
        let UserStatus::Idle = user_state.status else { return Err(RoomError::UserBusy)? };

        let room_id = RoomId(self.counter.fetch_add(1, Ordering::Relaxed));
        let mut room = Room::new(room_id);
        room.user_enter(user_clone)?;
        self.rooms.insert(room_id, room);
        user_state.status = UserStatus::InRoom(room_id);

        user.do_send(WsResponse::MoveToRoom(room_id));
        self.update_cached_rooms();
        Ok(room_id)
    }
}

struct JoinRoom {
    user: User,
    room_id: RoomId,
}

impl Handler<JoinRoom> for RoomManager {
    type Output = Result<()>;
    fn handle(&mut self, msg: JoinRoom, _ctx: &Context<Self>) -> Self::Output {
        let mut user_state = msg.user.state().write();
        let UserStatus::Idle = user_state.status else { return Err(RoomError::UserBusy)? };

        let room_id = msg.room_id;
        let room = self.get_mut(room_id)?;
        room.user_enter(msg.user.clone())?;
        user_state.status = UserStatus::InRoom(room_id);
        msg.user.do_send(WsResponse::MoveToRoom(room_id));
        room.broadcast_update();
        self.update_cached_rooms();
        Ok(())
    }
}

struct GetJoinedRoom {
    user_id: UserId,
    room_id: RoomId,
}

impl Handler<GetJoinedRoom> for RoomManager {
    type Output = Result<api::JoinedRoom>;
    fn handle(&mut self, msg: GetJoinedRoom, _ctx: &Context<Self>) -> Self::Output {
        let room = self.get_mut(msg.room_id)?;
        if room.get_user_mut(msg.user_id).is_none() {
            Err(RoomError::NotInRoom)?
        }
        Ok(room.to_joined_room())
    }
}

#[derive(Debug)]
struct UserRoomAction {
    user_id: UserId,
    room_id: RoomId,
    action: RoomAction,
}

impl Handler<UserRoomAction> for RoomManager {
    type Output = Result<()>;
    fn handle(&mut self, msg: UserRoomAction, _ctx: &Context<Self>) -> Self::Output {
        let room = self.get_mut(msg.room_id)?;
        room.room_action(msg.user_id, msg.action)
    }
}

impl RoomManager {
    fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            counter: AtomicI64::new(0),
            cached_rooms: Arc::new(RwLock::new(vec![])),
        }
    }

    fn get_mut(&mut self, room_id: RoomId) -> Result<&mut Room> {
        Ok(self
            .rooms
            .get_mut(&room_id)
            .ok_or(RoomError::RoomNotFound(room_id))?)
    }

    fn update_cached_rooms(&mut self) {
        *self.cached_rooms.write() = self.rooms.values().map(|room| room.to_api()).collect_vec()
    }
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
            users: self.users.iter().map(|user| user.to_api()).collect_vec(),
        }
    }

    fn get_user_mut(&mut self, user_id: UserId) -> Option<&mut RoomUser> {
        for user in &mut self.users {
            if user.user.id() == user_id {
                return Some(user);
            }
        }
        None
    }

    fn user_enter(&mut self, user: User) -> Result<()> {
        if self.users.len() >= 2 {
            Err(RoomError::RoomIsFull(self.id))?
        }
        self.users.push(RoomUser {
            user,
            is_ready: false,
        });
        Ok(())
    }

    fn all_users_ready(&self) -> bool {
        self.users.len() == 2 && self.users[0].is_ready && self.users[1].is_ready
    }

    fn start_game(&self) {
        let users = [&self.users[0].user, &self.users[1].user];
        let api_users = users.map(|u| u.to_api());
        let user_states = lock_both_user_states(users);
        let game = GameActor::new(users.map(|x| x.clone())).start();

        for mut state in user_states {
            state.status = UserStatus::InGame(game.clone());
        }
        for user in users {
            user.do_send(WsResponse::GameStart(GameInfo {
                game_id: game.id(),
                users: api_users.clone(),
                me: hexomino_core::Player::First,
            }));
        }
    }

    fn room_action(&mut self, user_id: UserId, action: RoomAction) -> Result<()> {
        let user = self.get_user_mut(user_id).ok_or(RoomError::NotInRoom)?;
        match action {
            RoomAction::Ready => {
                user.is_ready = true;
                if self.all_users_ready() {
                    self.start_game();
                }
            }
            RoomAction::UndoReady => {
                user.is_ready = false;
            }
        }
        self.broadcast_update();
        Ok(())
    }

    fn broadcast_update(&self) {
        let room = self.to_joined_room();
        for user in self.users.iter().map(|u| u.user.clone()) {
            user.do_send(WsResponse::RoomUpdate(room.clone()));
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

fn lock_both_user_states<'a>(users: [&'a User; 2]) -> [RwLockWriteGuard<'a, UserState>; 2] {
    let u0 = users[0];
    let u1 = users[1];
    if u0.id() < u1.id() {
        let s0 = u0.state().write();
        let s1 = u1.state().write();
        [s0, s1]
    } else {
        let s1 = u1.state().write();
        let s0 = u0.state().write();
        [s0, s1]
    }
}
