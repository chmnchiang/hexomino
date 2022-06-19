use std::time::Duration;

use api::{
    Api, JoinedRoom, MatchAction, MatchError, MatchHistoryNoGames, MatchState, Never, Room,
    RoomAction, RoomError, RoomId, StartWsApi, StartWsError, StartWsRequest, UserId, WsRequest, MatchToken,
};
use axum::extract::ws::{Message, WebSocket};
use once_cell::sync::OnceCell;
use tokio::{spawn, time::timeout};
use tracing::{debug, error, trace};

use crate::{
    auth::{authorize_jwt, Claims},
    result::ApiResult,
    DbPool,
};

use self::{
    match_history::{list_all_match_histories, list_user_match_histories},
    room::RoomManagerHandle,
    user::{User, UserPool, UserStatus},
};

pub mod actor;
pub mod deadline;
pub mod game;
pub mod match_history;
pub mod room;
pub mod user;

#[derive(Debug)]
enum KernelMsg {
    ConnectionLost(User),
    UserMessage(User, WsRequest),
}

pub struct Kernel {
    user_pool: UserPool,
    room_manager: RoomManagerHandle,
    db: DbPool,
}

async fn send_start_ws_error(mut ws: WebSocket, err: StartWsError) {
    let msg: <StartWsApi as Api>::Response = Err(err);
    if let Ok(buf) = bincode::serialize(&msg) {
        let _ = ws.send(Message::Binary(buf)).await;
    } else {
        debug!("failed to serialize StartWsResult: {:?}", msg.unwrap_err());
    }
}

static KERNEL: OnceCell<Kernel> = OnceCell::new();

impl Kernel {
    pub fn init(db: DbPool) {
        KERNEL
            .set(Self {
                room_manager: RoomManagerHandle::new(),
                user_pool: UserPool::new(db.clone()),
                db,
            })
            .map_err(|_| ())
            .expect("kernel is initialized twice");

        Kernel::spawn_services();
    }

    pub fn get() -> &'static Kernel {
        KERNEL.get().expect("kernel is not initialized")
    }

    #[tracing::instrument(skip_all)]
    pub async fn new_connection(&self, mut ws: WebSocket) {
        let result = authorize_ws(&mut ws).await;
        match result {
            Ok(claims) => self.user_pool.user_ws_connect(UserId(claims.id), ws).await,
            Err(err) => send_start_ws_error(ws, err).await,
        }
    }
}

impl Kernel {
    pub async fn get_user(&self, user_id: UserId) -> Option<User> {
        self.user_pool.get(user_id)
    }
    pub async fn get_room(&self, user: User) -> ApiResult<JoinedRoom, RoomError> {
        self.room_manager.get_joined_room(user).await
    }
    pub async fn list_rooms(&self) -> ApiResult<Vec<Room>, Never> {
        Ok(self.room_manager.list_rooms())
    }
    pub async fn join_room(&self, user: User, room_id: RoomId) -> ApiResult<(), RoomError> {
        self.room_manager.join_room(user, room_id).await
    }
    pub async fn create_room(&self, user: User) -> ApiResult<RoomId, RoomError> {
        self.room_manager.create_room(user).await
    }
    pub async fn create_or_join_match_room(
        &self,
        user: User,
        match_token: MatchToken,
    ) -> ApiResult<RoomId, RoomError> {
        self.room_manager.create_or_join_match_room(user, match_token).await
    }
    pub async fn leave_room(&self, user: User) -> ApiResult<(), RoomError> {
        self.room_manager.leave_room(user).await
    }
    pub async fn room_action(&self, user: User, action: RoomAction) -> ApiResult<(), RoomError> {
        self.room_manager.user_room_action(user, action).await
    }
    pub async fn match_action(&self, user: User, action: MatchAction) -> ApiResult<(), MatchError> {
        let game = {
            let UserStatus::InGame(game) = &user.state().read().status else {
                return Err(MatchError::NotInMatch)?
            };
            game.clone()
        };
        game.user_action(user, action).await
    }
    pub async fn sync_match(&self, user: User) -> ApiResult<MatchState, MatchError> {
        let game = {
            let UserStatus::InGame(game) = &user.state().read().status else {
                return Err(MatchError::NotInMatch)?
            };
            game.clone()
        };
        game.sync_match(user).await
    }
    pub async fn list_user_match_histories(
        &self,
        user: User,
    ) -> ApiResult<Vec<MatchHistoryNoGames>, Never> {
        if user.is_admin() {
            list_all_match_histories().await
        } else {
            list_user_match_histories(user.id()).await
        }
    }
}

const CHECK_USER_INTERVAL: Duration = Duration::from_secs(30);

impl Kernel {
    fn spawn_services() {
        spawn(async {
            let mut interval = tokio::time::interval(CHECK_USER_INTERVAL);
            loop {
                interval.tick().await;
                Kernel::get().user_pool.check_all_users();
            }
        });
    }

    async fn update(&self, message: KernelMsg) {
        use KernelMsg::*;
        trace!("update message = {:?}", message);
        match message {
            ConnectionLost(user) => user.drop_connection(),
            UserMessage(user, msg) => {
                self.handle_user_message(user, msg).await;
            }
        }
    }

    async fn handle_user_ws_message(&self, user: User, message: Message) {
        let msg = match message {
            Message::Binary(msg) => msg,
            Message::Close(_) => {
                self.update(KernelMsg::ConnectionLost(user)).await;
                return;
            }
            Message::Ping(_) => return,
            Message::Pong(_) => return,
            _ => {
                debug!("user send incorrect ws type");
                return;
            }
        };
        if let Ok(msg) = bincode::deserialize::<WsRequest>(&msg) {
            self.update(KernelMsg::UserMessage(user, msg)).await;
        } else {
            debug!("deserialize user data failed");
        }
    }

    async fn handle_user_message(&self, _user: User, msg: WsRequest) {
        match msg {}
    }
}

const WS_AUTH_TIMEOUT: Duration = Duration::from_secs(10);

async fn authorize_ws(ws: &mut WebSocket) -> Result<Claims, StartWsError> {
    use StartWsError::*;
    let recv_future = timeout(WS_AUTH_TIMEOUT, ws.recv());
    let result = recv_future.await.map_err(|_| Timeout)?;
    let result = result.ok_or(InitialHandshakeFailed)?;
    let result = result.map_err(|e| {
        error!("received error when authorizing websocket: {}", e);
        InternalError
    })?;
    if let Message::Binary(token) = result {
        let request =
            bincode::deserialize::<StartWsRequest>(&token).map_err(|_| InitialHandshakeFailed)?;
        Ok(authorize_jwt(&request.token).await.ok_or(WsAuthError)?)
    } else {
        Err(InitialHandshakeFailed)?
    }
}
