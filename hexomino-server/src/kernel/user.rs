use std::{
    future::{ready, Future},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Weak,
    },
};

use anyhow::anyhow;
use api::{Api, RoomId, StartWsApi, StartWsError, StartWsResponse, UserId, WsResult};
use axum::{
    async_trait,
    extract::{
        ws::{Message, WebSocket},
        FromRequest, RequestParts,
    },
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use dashmap::DashMap;
use derivative::Derivative;
use futures::{
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, StreamExt as _,
};
use getset::{CopyGetters, Getters};
use parking_lot::{RwLock, RwLockWriteGuard};
use stream_cancel::{StreamExt as _, TakeUntilIf, Trigger, Tripwire};
use tokio::{spawn, sync::Mutex};

use crate::{
    auth::{authorize_jwt, Claims},
    kernel::{send_start_ws_error, Kernel},
    result::CommonError,
};

#[cfg(feature = "competition-mode")]
use crate::DbPool;

use super::game::MatchHandle;

#[derive(Clone, Debug, derive_more::Deref)]
pub struct User(Arc<UserInner>);

impl User {
    fn on_connection_end(&self) {
        let state = self.state().write();
        if let UserStatus::InRoom(..) = state.status {
            drop(state);
            spawn(Kernel::get().room_manager.leave_room(self.clone()));
        }
    }
}

impl Eq for User {}
impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl User {
    pub fn lock_both_user_states(users: [&User; 2]) -> [RwLockWriteGuard<UserState>; 2] {
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
}

#[derive(Derivative, Getters, CopyGetters)]
#[derivative(Debug)]
pub struct UserInner {
    #[getset(get_copy = "pub")]
    id: UserId,
    #[getset(get = "pub")]
    data: UserData,
    #[getset(get = "pub")]
    state: RwLock<UserState>,
    #[derivative(Debug = "ignore")]
    #[getset(get = "pub")]
    connection: Connection,
}

#[derive(Debug)]
pub struct UserData {
    pub username: String,
    pub name: String,
}

#[derive(Debug)]
pub struct UserState {
    pub status: UserStatus,
}

#[derive(Debug)]
pub enum UserStatus {
    Idle,
    InRoom(RoomId),
    InGame(MatchHandle),
}

type WsStream = TakeUntilIf<SplitStream<WebSocket>, Tripwire>;

pub struct Connection {
    inner: RwLock<Option<ConnectionInner>>,
}

struct ConnectionInner {
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    _recv_trigger: Trigger,
}

impl ConnectionInner {
    fn new(ws: WebSocket) -> (Self, WsStream) {
        let (sender, receiver) = ws.split();
        let (trigger, tripwire) = Tripwire::new();
        let receiver = receiver.take_until_if(tripwire);
        (
            ConnectionInner {
                sender: Arc::new(Mutex::new(sender)),
                _recv_trigger: trigger,
            },
            receiver,
        )
    }
}

impl Connection {
    fn new() -> Self {
        Self {
            inner: RwLock::new(None),
        }
    }

    fn set(&self, ws: WebSocket) -> WsStream {
        let (inner, stream) = ConnectionInner::new(ws);
        *self.inner.write() = Some(inner);
        stream
    }

    fn drop(&self) {
        *self.inner.write() = None;
    }

    fn send(&self, msg: Message) -> impl Future<Output = anyhow::Result<()>> {
        let connection = &self.inner.read();
        if let Some(connection) = connection.as_ref() {
            let sender = connection.sender.clone();
            async move {
                let mut sender = sender.lock().await;
                if let Err(err) = sender.send(msg).await {
                    let err = anyhow!("Failed to send websocket to user: {:?}", err);
                    Err(err)
                } else {
                    Ok(())
                }
            }
            .left_future()
        } else {
            async { Err(anyhow!("Connection is not established")) }.right_future()
        }
    }
}

impl UserInner {
    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn username(&self) -> &str {
        &self.data.username
    }

    pub fn to_api(&self) -> api::User {
        api::User {
            id: self.id,
            name: self.name().to_string(),
        }
    }

    pub fn is_admin(&self) -> bool {
        // TODO: Proper way of checking admin
        self.username() == "admin"
    }

    pub fn drop_connection(&self) {
        self.connection.drop();
    }

    pub fn send(&self, resp: WsResult) -> impl Future<Output = anyhow::Result<()>> {
        tracing::trace!(
            "Send to user={} Websocket message = {resp:?}",
            self.username()
        );
        match bincode::serialize(&resp) {
            Ok(bytes) => self.connection.send(Message::Binary(bytes)).left_future(),
            Err(err) => {
                tracing::error!("cannot serialize message {resp:?} for sending: {err}");
                ready(Err(anyhow::anyhow!("cannot serialize message for sending"))).right_future()
            }
        }
    }

    pub fn do_send(&self, resp: WsResult) {
        spawn(self.send(resp).map(|_| ()));
    }

    pub fn send_status_update(&self) {
        self.do_send(api::WsResponse::UserStatusUpdate(
            self.state().read().status.to_api(),
        ));
    }

    pub fn do_send_ping(&self) {
        spawn(self.connection().send(Message::Ping(vec![])));
    }
}

impl UserData {
    #[cfg(feature = "competition-mode")]
    async fn fetch(db: &DbPool, UserId(id): UserId) -> Option<Self> {
        let user = sqlx::query!(
            r#"
            SELECT username, name FROM Users
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(db)
        .await
        .ok()?;

        Some(Self {
            username: user.username,
            name: unwrap_name_or_unnamed(user.name),
        })
    }
}

pub fn unwrap_name_or_unnamed(name: Option<String>) -> String {
    name.unwrap_or_else(|| "<Unnamed>".to_string())
}

#[cfg(feature = "competition-mode")]
pub struct UserPool {
    db: DbPool,
    users: DashMap<UserId, Weak<UserInner>>,
    prev_len: AtomicUsize,
}

#[cfg(not(feature = "competition-mode"))]
pub struct UserPool {
    users: DashMap<String, Weak<UserInner>>,
    prev_len: AtomicUsize,
}

impl UserPool {
    #[cfg(feature = "competition-mode")]
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            users: DashMap::new(),
            prev_len: AtomicUsize::new(0),
        }
    }

    #[cfg(not(feature = "competition-mode"))]
    pub fn new() -> Self {
        Self {
            users: DashMap::new(),
            prev_len: AtomicUsize::new(0),
        }
    }

    #[cfg(feature = "competition-mode")]
    pub fn get(&self, id: UserId) -> Option<User> {
        use dashmap::mapref::entry::Entry::*;
        match self.users.entry(id) {
            Occupied(occupied) => match occupied.get().upgrade() {
                Some(user) => Some(User(user)),
                None => {
                    occupied.remove();
                    None
                }
            },
            Vacant(_) => None,
        }
    }

    #[cfg(not(feature = "competition-mode"))]
    pub fn get(&self, username: String) -> Option<User> {
        use dashmap::mapref::entry::Entry::*;
        match self.users.entry(username) {
            Occupied(occupied) => match occupied.get().upgrade() {
                Some(user) => Some(User(user)),
                None => {
                    occupied.remove();
                    None
                }
            },
            Vacant(_) => None,
        }
    }

    pub fn check_all_users(&self) {
        type ShouldKeep = bool;
        fn user_check(user: &Weak<UserInner>) -> ShouldKeep {
            if let Some(user) = Weak::upgrade(user) {
                user.do_send_ping();
                true
            } else {
                false
            }
        }
        self.users.retain(|_, user| user_check(user));
        let cur_len = self.users.len();
        if self.prev_len.load(Ordering::Relaxed) != cur_len {
            self.prev_len.store(cur_len, Ordering::Relaxed);
            tracing::info!("users size = {}", cur_len);
        }
    }

    #[cfg(feature = "competition-mode")]
    pub async fn user_ws_connect(&self, id: UserId, ws: WebSocket) {
        let user = if let Some(user) = self.get(id) {
            user
        } else {
            let data = match UserData::fetch(&self.db, id).await {
                None => {
                    tracing::debug!("user id={} does not exists", id);
                    send_start_ws_error(ws, StartWsError::WsAuthError).await;
                    return;
                }
                Some(data) => data,
            };
            let user = UserInner {
                id,
                data,
                state: RwLock::new(UserState {
                    status: UserStatus::Idle,
                }),
                connection: Connection::new(),
            };

            User(Arc::new(user))
        };
        let ws_stream = user.connection.set(ws);
        spawn(connection_recv_loop(user.clone(), ws_stream));
        self.users.insert(user.id(), Arc::downgrade(&user.0));

        let msg: <StartWsApi as Api>::Response = Ok(StartWsResponse {
            username: user.name().to_string(),
        });
        match bincode::serialize(&msg) {
            Ok(buf) => {
                let _ = user.connection().send(Message::Binary(buf)).await;
                tracing::debug!("User connection complete.");
                user.send_status_update();
            }
            Err(err) => {
                tracing::error!(
                    "failed to serialize StartWsResult {:?}: {err}",
                    msg.unwrap_err()
                );
            }
        }
    }

    #[cfg(not(feature = "competition-mode"))]
    pub async fn user_ws_connect(&self, claims: Claims, ws: WebSocket) {
        use uuid::Uuid;

        let user = if let Some(user) = self.get(claims.username.clone()) {
            user
        } else {
            let data = UserData {
                username: claims.username.clone(),
                name: claims.username.clone(),
            };
            let user = UserInner {
                id: UserId(claims.id),
                data,
                state: RwLock::new(UserState {
                    status: UserStatus::Idle,
                }),
                connection: Connection::new(),
            };

            User(Arc::new(user))
        };
        let ws_stream = user.connection.set(ws);
        spawn(connection_recv_loop(user.clone(), ws_stream));
        self.users.insert(claims.username, Arc::downgrade(&user.0));

        let msg: <StartWsApi as Api>::Response = Ok(StartWsResponse {
            username: user.name().to_string(),
        });
        match bincode::serialize(&msg) {
            Ok(buf) => {
                let _ = user.connection().send(Message::Binary(buf)).await;
                tracing::debug!("User connection complete.");
                user.send_status_update();
            }
            Err(err) => {
                tracing::error!(
                    "failed to serialize StartWsResult {:?}: {err}",
                    msg.unwrap_err()
                );
            }
        }
    }
}

#[tracing::instrument(skip_all, fields(user = ?user.username()))]
async fn connection_recv_loop(user: User, mut receiver: WsStream) {
    tracing::debug!("User receive loop started.");
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
                Kernel::get()
                    .handle_user_ws_message(user.clone(), msg)
                    .await;
            }
            Err(_) => {
                user.drop_connection();
            }
        }
    }
    tracing::debug!("User receive loop ended.");
    user.on_connection_end();
}

impl UserStatus {
    fn to_api(&self) -> api::UserStatus {
        use UserStatus::*;
        match self {
            Idle => api::UserStatus::Idle,
            InRoom(..) => api::UserStatus::InRoom,
            InGame(..) => api::UserStatus::InGame,
        }
    }
}

#[async_trait]
impl<B: Send> FromRequest<B> for User {
    type Rejection = CommonError;

    #[cfg(feature = "competition-mode")]
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| CommonError::Unauthorized)?;

        let claims = authorize_jwt(bearer.token())
            .await
            .ok_or(CommonError::Unauthorized)?;
        Kernel::get()
            .get_user(UserId(claims.id))
            .await
            .ok_or(CommonError::Unauthorized)
    }

    #[cfg(not(feature = "competition-mode"))]
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| CommonError::Unauthorized)?;

        let claims = authorize_jwt(bearer.token())
            .await
            .ok_or(CommonError::Unauthorized)?;
        Kernel::get()
            .get_user(claims.username)
            .await
            .ok_or(CommonError::Unauthorized)
    }
}
