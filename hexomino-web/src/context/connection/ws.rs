use std::{
    cell::RefCell,
    future::Future,
    mem::{self},
    pin::Pin,
    rc::{Rc, Weak},
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{anyhow, Context as _};
use api::{Api, StartWsApi, StartWsError, StartWsRequest, WsResult};
use futures::{
    stream::{SplitSink, SplitStream, TakeUntil},
    SinkExt, StreamExt,
};
use gloo::{
    net::websocket::{futures::WebSocket, Message},
    utils::document,
};

use wasm_bindgen_futures::spawn_local;
use yew::Callback;

use crate::{
    context::ConnectionError,
    util::{Mutex, ResultExt},
};

use super::Result;

#[derive(Default)]
pub struct WsConnection {
    inner: RefCell<Option<Rc<WsConnectionInner>>>,
    bus: Rc<WsMessageBus>,
}

impl PartialEq for WsConnection {
    fn eq(&self, other: &Self) -> bool {
        self.connected() == other.connected()
    }
}
impl Eq for WsConnection {}

type WsSender = SplitSink<WebSocket, Message>;
type WsReceiver = TakeUntil<SplitStream<WebSocket>, Tripwire>;
pub type WsCallback = Callback<Rc<WsResult>>;

impl WsConnection {
    pub async fn connect(
        &self,
        token: String,
        error_callback: Callback<ConnectionError>,
    ) -> Result<()> {
        let host = document()
            .location()
            .context("cannot get location")?
            .host()
            .map_err(|_| anyhow!("cannot get host"))?;
        let mut ws = WebSocket::open(&format!("ws://{host}/ws")).anyhow()?;
        let result: Result<_> = (|| async {
            let msg = bincode::serialize(&StartWsRequest { token }).anyhow()?;
            ws.send(Message::Bytes(msg)).await.anyhow()?;

            let msg = ws
                .next()
                .await
                .context("fail to get start ws response from server")?
                .anyhow()?;
            if let Message::Bytes(msg) = msg {
                let msg = bincode::deserialize::<<StartWsApi as Api>::Response>(msg.as_slice())
                    .anyhow()?;
                match msg {
                    Ok(x) => Ok(x),
                    Err(StartWsError::WsAuthError) => Err(ConnectionError::Unauthorized),
                    Err(err) => Err(err).anyhow()?,
                }
            } else {
                Err(anyhow!("server send message with wrong ws type: {msg:?}"))?
            }
        })()
        .await;

        if result.is_ok() {
            self.setup_connection(ws, self.bus.clone(), error_callback);
        } else {
            mem::forget(ws);
        }
        result.map(|_| ())
    }

    pub fn connected(&self) -> bool {
        self.inner.borrow().is_some()
    }

    pub fn disconnect(&self) {
        *self.inner.borrow_mut() = None;
    }

    pub fn register_callback(&self, callback: WsCallback) -> WsListenerToken {
        self.bus.clone().register(callback)
    }

    fn setup_connection(
        &self,
        ws: WebSocket,
        bus: Rc<WsMessageBus>,
        error_callback: Callback<ConnectionError>,
    ) {
        let (sender, receiver) = ws.split();
        let (trigger, tripwire) = create_tripwire();
        let connection = WsConnectionInner {
            sender: Mutex::new(sender),
            error_callback: error_callback.clone(),
            _trigger: trigger,
        };
        spawn_receive_loop(receiver.take_until(tripwire), bus, error_callback);
        *self.inner.borrow_mut() = Some(Rc::new(connection));
    }

    pub async fn send(self: Rc<Self>, msg: api::WsRequest) -> Result<()> {
        let connection = self
            .inner
            .borrow()
            .as_ref()
            .ok_or_else(|| anyhow!("connection is not set up when sending messages"))?
            .clone();
        let msg = bincode::serialize(&msg).anyhow()?;
        let mut sender = connection.sender.lock().await;
        sender
            .send(Message::Bytes(msg))
            .await
            .map_err(|_| ConnectionError::WsConnectionClose)?;
        Ok(())
    }
}

struct WsConnectionInner {
    sender: Mutex<WsSender>,
    error_callback: Callback<ConnectionError>,
    _trigger: Trigger,
}

fn spawn_receive_loop(
    mut receiver: WsReceiver,
    bus: Rc<WsMessageBus>,
    error_callback: Callback<ConnectionError>,
) {
    spawn_local(async move {
        log::debug!("Websocket receive loop started.");
        loop {
            let receiver = &mut receiver;
            let result: Result<_> = (|| async move {
                let msg = receiver
                    .next()
                    .await
                    .ok_or(ConnectionError::WsConnectionClose)?
                    .map_err(|_| ConnectionError::WsConnectionClose)?;
                if let Message::Bytes(msg) = msg {
                    let msg = bincode::deserialize::<WsResult>(&msg)
                        .context("deserialize server message failed")?;
                    Ok(msg)
                } else {
                    Err(anyhow!("server send message with wrong ws type: {msg:?}"))?
                }
            })()
            .await;

            match result {
                Ok(x) => bus.broadcast(x),
                Err(err) => {
                    error_callback.emit(err);
                    break;
                }
            }
        }
        // TODO: gloo's websocket closure does not work well when the Websocket struct is dropped.
        // we have no choice to allow some leak memory here.
        log::debug!("Websocket receive loop ended.");
    })
}

type Tripwire = Pin<Box<dyn Future<Output = ()>>>;

fn create_tripwire() -> (Trigger, Tripwire) {
    let lock = Rc::new(Mutex::new(()));
    let trigger = Trigger { lock: lock.clone() };
    lock.raw_lock();
    let fut = async move {
        lock.lock().await;
    };
    (trigger, Box::pin(fut))
}

struct Trigger {
    lock: Rc<Mutex<()>>,
}

impl Trigger {
    fn new(lock: Rc<Mutex<()>>) -> Self {
        lock.raw_lock();
        Self { lock }
    }
}

impl Drop for Trigger {
    fn drop(&mut self) {
        self.lock.raw_unlock();
    }
}

#[derive(Default)]
struct WsMessageBus {
    listeners: RefCell<Vec<(WsListenerId, WsCallback)>>,
}

pub struct WsListenerToken {
    id: WsListenerId,
    bus: Weak<WsMessageBus>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct WsListenerId(usize);

impl WsMessageBus {
    pub fn register(self: Rc<Self>, callback: WsCallback) -> WsListenerToken {
        let id = WsListenerId::new();
        self.listeners.borrow_mut().push((id, callback));
        WsListenerToken {
            id,
            bus: Rc::downgrade(&self),
        }
    }

    fn deregister(&self, id: WsListenerId) {
        let mut listeners = self.listeners.borrow_mut();
        if let Some(index) = listeners.iter().position(|(jd, _)| id == *jd) {
            listeners.swap_remove(index);
        }
    }

    pub fn broadcast(&self, msg: WsResult) {
        let msg = Rc::new(msg);
        let listeners = {
            let listeners = self.listeners.borrow().clone();
            listeners
        };
        for (_, listener) in listeners {
            listener.emit(msg.clone())
        }
    }
}

impl WsListenerId {
    fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        WsListenerId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Drop for WsListenerToken {
    fn drop(&mut self) {
        if let Some(bus) = self.bus.upgrade() {
            bus.deregister(self.id)
        }
    }
}
