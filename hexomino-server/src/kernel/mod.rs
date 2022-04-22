use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use api::{cerr, RoomId, RoomMsg, UserId};
use axum::extract::ws::{Message, WebSocket};
use tokio::time::timeout;
use tracing::{debug, error, info, trace};

use crate::{
    http::{authorize_jwt, Claims},
    DbPool,
};

use self::{
    room::RoomManager,
    user::{User, UserPool},
};

mod room;
mod user;

#[derive(Debug)]
enum KernelMsg {
    ConnectionLost(User),
    UserMessage(User, api::Request),
}

pub struct Kernel {
    user_pool: UserPool,
    room_manager: RoomManager,
}

async fn ws_send_api_error(mut ws: WebSocket, err: api::Error) {
    if let Ok(buf) = bincode::serialize(&err) {
        let _ = ws.send(Message::Binary(buf)).await;
    } else {
        error!("Failed to serialize error message = {:?}", err);
    }
}

impl Kernel {
    pub fn new(db: DbPool) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            room_manager: RoomManager::new(),
            user_pool: UserPool::new(me.clone(), db),
        })
    }

    pub async fn new_connection(self: Arc<Kernel>, mut ws: WebSocket) {
        debug!("new connection");
        let result = authorize_ws(&mut ws).await;
        match result {
            Ok(claims) => self.user_pool.user_ws_connect(UserId(claims.id), ws).await,
            Err(err) => ws_send_api_error(ws, err).await,
        }
    }
}

const WS_AUTH_TIMEOUT: Duration = Duration::from_secs(10);

async fn authorize_ws(ws: &mut WebSocket) -> api::Result<Claims> {
    let recv_future = timeout(WS_AUTH_TIMEOUT, ws.recv());
    let result = recv_future
        .await
        .map_err(|_| cerr!("Start connection timeout"))?;
    let result = result.ok_or_else(|| cerr!("Start connection failed, no message"))?;
    let result = result.map_err(|e| anyhow!(e))?;
    if let Message::Text(token) = result {
        match authorize_jwt(&token).await {
            Err(_) => Err(cerr!("Authenticate error")),
            Ok(claims) => Ok(claims),
        }
    } else {
        Err(cerr!("Start connection failed, wrong message type"))
    }
}

impl Kernel {
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
            Message::Ping(_) | Message::Pong(_) => return,
            _ => {
                info!("user send incorrect ws type");
                return;
            }
        };
        if let Ok(msg) = bincode::deserialize::<api::Request>(&msg) {
            self.update(KernelMsg::UserMessage(user, msg));
        } else {
            info!("deserialize user data failed");
        }
    }

    async fn handle_user_message(&self, user: User, msg: api::Request) {
        use api::Request::*;
        match msg {
            GetRooms => {
                self.get_rooms(user).await;
            }
            JoinRoom(room_id) => {
                self.join_room(user, room_id).await;
            }
            CreateRoom => {
                self.create_room(user).await;
            }
            _ => {
                todo!();
            }
        }
    }
}

impl Kernel {
    async fn get_rooms(&self, user: User) {
        let rooms = self.room_manager.get_rooms();
        let resp = api::Response::RoomMsg(RoomMsg::SyncRooms(rooms));
        user.send(resp).await;
    }
    async fn join_room(&self, user: User, room_id: RoomId) {
        let room = self.room_manager.join_room(user.clone(), room_id);
        let resp = room.map(|room| api::Response::RoomMsg(RoomMsg::JoinRoom(room)));
        user.send_result(resp).await;
    }
    async fn create_room(&self, user: User) {
        let room = self.room_manager.create_room(user.clone());
        let resp = room.map(|room| api::Response::RoomMsg(RoomMsg::JoinRoom(room)));
        user.send_result(resp).await;
    }
}
