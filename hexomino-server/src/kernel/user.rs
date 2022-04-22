use std::{
    future::Future,
    ops::Deref,
    sync::{Arc, Weak},
};

use anyhow::anyhow;
use api::{cerr, HelloFromKernel, UserId};
use axum::extract::ws::{Message, WebSocket};
use derivative::Derivative;
use futures::{
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, StreamExt as _,
};
use parking_lot::RwLock;
use stream_cancel::{StreamExt as _, TakeUntilIf, Trigger, Tripwire};
use tokio::{spawn, sync::Mutex};
use tracing::debug;

use crate::DbPool;

use super::{ws_send_api_error, Kernel};

#[derive(Clone, Debug)]
pub struct User(Arc<UserInner>);

impl Deref for User {
    type Target = Arc<UserInner>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Eq for User {}
impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl From<User> for api::User {
    fn from(user: User) -> Self {
        user.into()
    }
}

impl From<&User> for api::User {
    fn from(user: &User) -> Self {
        Self { id: user.id, name: user.name() }
    }
}


#[derive(Derivative)]
#[derivative(Debug)]
pub struct UserInner {
    id: UserId,
    data: RwLock<UserData>,
    state: RwLock<UserState>,
    #[derivative(Debug = "ignore")]
    connection: Connection,
}

#[derive(Debug)]
pub struct UserData {
    name: String,
}

#[derive(Debug)]
pub struct UserState;

type CancellableStream<T> = TakeUntilIf<SplitStream<T>, Tripwire>;

struct Connection {
    inner: RwLock<Option<ConnectionInner>>,
}

struct ConnectionInner {
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    _recv_trigger: Trigger,
}

impl Connection {
    fn new(ws: WebSocket) -> (Self, CancellableStream<WebSocket>) {
        let (sender, receiver) = ws.split();
        let (trigger, tripwire) = Tripwire::new();
        let receiver = receiver.take_until_if(tripwire);
        (
            Connection {
                inner: RwLock::new(Some(ConnectionInner {
                    sender: Arc::new(Mutex::new(sender)),
                    _recv_trigger: trigger,
                })),
            },
            receiver,
        )
    }

    fn drop(&self) {
        let x = self.inner.write();
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
}

impl UserInner {
    fn name(&self) -> String {
        self.data.read().name.clone()
    }

    pub fn drop_connection(&self) {
        self.connection.drop();
    }

    pub fn send(&self, resp: api::Response) -> impl Future<Output = anyhow::Result<()>> {
        self.connection.send(Message::Binary(
            bincode::serialize(&resp).expect(&format!("cannot serialzie {resp:?}")),
        ))
    }

    pub fn send_result(
        &self,
        resp: api::Result<api::Response>,
    ) -> impl Future<Output = anyhow::Result<()>> {
        match resp {
            Ok(resp) => self.send(resp),
            Err(err) => self.send(api::Response::Error(err)),
        }
    }
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

pub struct UserPool {
    kernel: Weak<Kernel>,
    db: DbPool,
}

impl UserPool {
    pub fn new(kernel: Weak<Kernel>, db: DbPool) -> Self {
        Self { kernel, db }
    }

    fn kernel(&self) -> Arc<Kernel> {
        self.kernel.upgrade().expect("kernel is not valid")
    }

    pub async fn user_ws_connect(&self, id: UserId, ws: WebSocket) {
        let data = match UserData::fetch(&self.db, id).await {
            None => return ws_send_api_error(ws, cerr!("User does not exists")).await,
            Some(data) => data,
        };
        let (connection, receiver) = Connection::new(ws);
        let user = UserInner {
            id,
            data: RwLock::new(data),
            state: RwLock::new(UserState),
            connection,
        };
        let user = User(Arc::new(user));
        spawn(self.connection_recv_loop(user.clone(), receiver));
        let fut = user.connection.send(Message::Binary(
            bincode::serialize(&HelloFromKernel {
                username: user.name(),
            })
            .unwrap(),
        ));
        fut.await;
    }

    fn connection_recv_loop(
        &self,
        user: User,
        mut receiver: CancellableStream<WebSocket>,
    ) -> impl Future<Output = ()> {
        let kernel = self.kernel();
        async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(msg) => {
                        kernel.handle_user_ws_message(user.clone(), msg).await;
                    }
                    Err(_) => {
                        user.drop_connection();
                    }
                }
            }
        }
    }
}
