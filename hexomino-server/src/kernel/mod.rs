use std::{future::Future, sync::Arc, time::Duration};

use anyhow::anyhow;
use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, StreamExt as _,
};
use hexomino_api::{cerr, Error as ApiError, HelloFromKernel};
use parking_lot::RwLock;
use stream_cancel::{StreamExt as _, TakeUntilIf, Trigger, Tripwire};
use tokio::{spawn, sync::Mutex, time::timeout};
use tracing::{debug, error, trace};

use crate::{api::authorize_jwt, DbPool};

type Shared<T> = Arc<RwLock<T>>;

enum KernelMsg {
    ConnectionLost,
    Message(Message),
}

pub struct Kernel {
    db: DbPool,
    //users: UserPool,
    rooms: RoomManager,
}

async fn ws_send_api_error(mut ws: WebSocket, err: ApiError) {
    if let Ok(buf) = bincode::serialize(&err) {
        let _ = ws.send(Message::Binary(buf)).await;
    } else {
        error!("Failed to serialize error message = {:?}", err);
    }
}

impl Kernel {
    pub async fn new_connection(self: Arc<Kernel>, mut ws: WebSocket) {
        debug!("new connection");
        let recv_future = timeout(Duration::from_secs(10), ws.recv());
        let run = || async {
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
        };

        let result = run().await;
        match result {
            Ok(claims) => self.check_in_user(UserId(claims.id), ws).await,
            Err(err) => ws_send_api_error(ws, err).await,
        }
    }

    async fn check_in_user(self: Arc<Kernel>, id: UserId, ws: WebSocket) {
        let data = match UserData::fetch(&self.db, id).await {
            None => return ws_send_api_error(ws, cerr!("User does not exists")).await,
            Some(data) => data,
        };
        let (connection, receiver) = Connection::new(ws);
        let user = Arc::new(RwLock::new(User {
            id,
            data,
            connection: Some(connection),
        }));
        spawn(self.connection_recv_loop(user.clone(), receiver));
        let name = user.read().data.name.clone();
        let fut = user.read().send(Message::Binary(
            bincode::serialize(&HelloFromKernel { username: name }).unwrap(),
        ));
        fut.await;
        trace!("user checked in");
    }

    async fn connection_recv_loop(
        self: Arc<Self>,
        user: Shared<User>,
        mut receiver: CancellableStream<WebSocket>,
    ) {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(msg) => {
                    self.handle_message(Arc::clone(&user), KernelMsg::Message(msg));
                }
                Err(_) => {
                    user.write().drop_connection();
                }
            }
        }
    }
}

impl Kernel {
    pub fn new(db: DbPool) -> Arc<Self> {
        Arc::new(Self {
            db,
            rooms: RoomManager {},
        })
    }

    fn handle_message(&self, user: Shared<User>, message: KernelMsg) {
        use KernelMsg::*;
        match message {
            ConnectionLost => user.write().drop_connection(),
            Message(msg) => {
                println!("recv = {:?}", msg);
            }
        }
    }
}

type CancellableStream<T> = TakeUntilIf<SplitStream<T>, Tripwire>;

struct Connection {
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    recv_trigger: Trigger,
}

impl Connection {
    fn new(ws: WebSocket) -> (Self, CancellableStream<WebSocket>) {
        let (sender, receiver) = ws.split();
        let (trigger, tripwire) = Tripwire::new();
        let receiver = receiver.take_until_if(tripwire);
        (
            Connection {
                sender: Arc::new(Mutex::new(sender)),
                recv_trigger: trigger,
            },
            receiver,
        )
    }
}

#[derive(Clone, Copy)]
struct UserId(i64);

struct User {
    id: UserId,
    data: UserData,
    connection: Option<Connection>,
}

impl User {
    fn drop_connection(&mut self) {
        self.connection = None;
    }

    fn send(&self, msg: Message) -> impl Future + Send {
        if let Some(connection) = &self.connection {
            let sender = connection.sender.clone();
            async move {
                let mut sender = sender.lock().await;
                if let Err(err) = sender.send(msg).await {
                    let err = anyhow!("Failed to send websocket to user: {:?}", err);
                    debug!("{:?}", err);
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

    fn spawn_send(&self, msg: Message) {
        spawn(self.send(msg).map(|_| ()));
    }
}

struct UserData {
    name: String,
}

impl UserData {
    async fn fetch(db: &DbPool, UserId(id): UserId) -> Option<Self> {
        let user = sqlx::query!(
            r#"
            SELECT name FROM Users
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(db)
        .await
        .ok()?;

        Some(Self { name: user.name })
    }
}

struct UserPool {}

impl UserPool {
    fn new() -> Self {
        todo!();
    }

    fn socket_connect(id: String, socket: WebSocket) {
        todo!();
    }
}

struct RoomManager {}
