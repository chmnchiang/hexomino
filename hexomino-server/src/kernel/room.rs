use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::Duration,
};

use api::{MatchConfig, RoomAction, RoomError, RoomId, UserId, WsResponse};
use itertools::Itertools;
use parking_lot::RwLock;

use crate::{
    kernel::{user::UserStatus, User},
    result::ApiResult,
};

use super::{
    actor::{Actor, Addr, Context, Handler},
    game::{MatchActor, to_match_settings},
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
    pub async fn leave_room(&self, user: User) -> Result<()> {
        self.addr.send(LeaveRoom { user }).await
    }
    pub async fn get_joined_room(&self, user: User) -> Result<api::JoinedRoom> {
        self.addr.send(GetJoinedRoom { user }).await
    }
    pub async fn user_room_action(&self, user: User, action: RoomAction) -> Result<()> {
        self.addr.send(UserRoomAction { user, action }).await
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
    config: MatchConfig,
}

const CACHED_ROOMS_UPDATE_INTERVAL: u64 = 3;

impl Actor for RoomManager {
    fn started(&mut self, ctx: &Context<Self>) {
        ctx.notify_later(
            UpdateCachedRooms,
            Duration::from_secs(CACHED_ROOMS_UPDATE_INTERVAL),
        );
    }
}

#[derive(Debug)]
struct CreateRoom {
    user: User,
}

impl Handler<CreateRoom> for RoomManager {
    type Output = Result<RoomId>;

    #[tracing::instrument(skip_all, fields(action = "CreateRoom", user = ?msg.user.username()), ret)]
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

        drop(user_state);
        user.send_status_update();
        Ok(room_id)
    }
}

#[derive(Debug)]
struct JoinRoom {
    user: User,
    room_id: RoomId,
}

impl Handler<JoinRoom> for RoomManager {
    type Output = Result<()>;

    #[tracing::instrument(skip_all, fields(action = "JoinRoom", user = ?msg.user.username(), room = ?msg.room_id), ret)]
    fn handle(&mut self, msg: JoinRoom, _ctx: &Context<Self>) -> Self::Output {
        let mut user_state = msg.user.state().write();
        let UserStatus::Idle = user_state.status else { return Err(RoomError::UserBusy)? };

        let room_id = msg.room_id;
        let room = self.get_mut(room_id)?;
        room.user_enter(msg.user.clone())?;
        user_state.status = UserStatus::InRoom(room_id);
        drop(user_state);
        msg.user.send_status_update();
        room.broadcast_update();
        Ok(())
    }
}

#[derive(Debug)]
struct LeaveRoom {
    user: User,
}

impl Handler<LeaveRoom> for RoomManager {
    type Output = Result<()>;

    #[tracing::instrument(skip_all, fields(action = "LeaveRoom", user = ?msg.user.username()), ret)]
    fn handle(&mut self, msg: LeaveRoom, _ctx: &Context<Self>) -> Self::Output {
        let user_id = msg.user.id();
        let mut user_state = msg.user.state().write();
        let UserStatus::InRoom(room_id) = user_state.status
            else { return Err(RoomError::NotInRoom.into()); };
        let room = self.get_mut(room_id)?;

        room.users.retain(|u| u.user.id() != user_id);
        user_state.status = UserStatus::Idle;
        drop(user_state);
        msg.user.send_status_update();

        if room.users.is_empty() {
            self.remove_room(room_id);
        } else {
            room.broadcast_update();
        }
        Ok(())
    }
}

struct GetJoinedRoom {
    user: User,
}

impl Handler<GetJoinedRoom> for RoomManager {
    type Output = Result<api::JoinedRoom>;
    fn handle(&mut self, msg: GetJoinedRoom, _ctx: &Context<Self>) -> Self::Output {
        let user_state = msg.user.state().read();
        let UserStatus::InRoom(room_id) = user_state.status
            else { return Err(RoomError::NotInRoom.into()); };
        let room = self.get_mut(room_id)?;
        if room.get_user_mut(msg.user.id()).is_none() {
            return Err(RoomError::NotInRoom.into());
        }
        Ok(room.to_joined_room())
    }
}

#[derive(Debug)]
struct UserRoomAction {
    user: User,
    action: RoomAction,
}

impl Handler<UserRoomAction> for RoomManager {
    type Output = Result<()>;
    #[tracing::instrument(skip_all, fields(action = "UserRoomAction", user = ?msg.user.username(), action=?msg.action), ret)]
    fn handle(&mut self, msg: UserRoomAction, _ctx: &Context<Self>) -> Self::Output {
        let user_state = msg.user.state().read();
        let UserStatus::InRoom(room_id) = user_state.status
            else { return Err(RoomError::NotInRoom.into()); };
        let room = self.get_mut(room_id)?;
        drop(user_state);

        let should_remove = room.room_action(msg.user.id(), msg.action)?;
        if should_remove {
            self.remove_room(room_id);
        }
        Ok(())
    }
}

#[derive(Debug)]
struct UpdateCachedRooms;

impl Handler<UpdateCachedRooms> for RoomManager {
    type Output = ();
    fn handle(&mut self, _msg: UpdateCachedRooms, ctx: &Context<Self>) -> Self::Output {
        self.update_cached_rooms();
        ctx.notify_later(
            UpdateCachedRooms,
            Duration::from_secs(CACHED_ROOMS_UPDATE_INTERVAL),
        );
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

    fn remove_room(&mut self, room_id: RoomId) {
        self.rooms.remove(&room_id);
    }
}

impl Room {
    fn new(id: RoomId) -> Self {
        Self {
            id,
            users: vec![],
            config: MatchConfig::Normal,
        }
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
            settings: to_match_settings(self.config),
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
        let user_states = User::lock_both_user_states(users);
        let game = MatchActor::new(users.map(|x| x.clone()), self.config).start();
        tracing::info!("game start: id = {}", game.id());

        for mut state in user_states {
            state.status = UserStatus::InGame(game.clone());
        }

        for user in users {
            user.send_status_update();
        }
    }

    fn room_action(&mut self, user_id: UserId, action: RoomAction) -> Result<ShouldRemove> {
        let user = self.get_user_mut(user_id).ok_or(RoomError::NotInRoom)?;
        let mut should_remove = false;
        match action {
            RoomAction::Ready => {
                user.is_ready = true;
                if self.all_users_ready() {
                    self.start_game();
                    should_remove = true;
                }
            }
            RoomAction::UndoReady => {
                user.is_ready = false;
            }
            RoomAction::SetConfig(config) => {
                self.config = config;
            }
        }
        self.broadcast_update();
        Ok(should_remove)
    }

    fn broadcast_update(&self) {
        let room = self.to_joined_room();
        for user in self.users.iter().map(|u| u.user.clone()) {
            user.do_send(WsResponse::RoomUpdate(room.clone()));
        }
    }
}

type ShouldRemove = bool;

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
