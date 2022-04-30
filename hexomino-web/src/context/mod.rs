use std::{future::Future, rc::Rc};

use anyhow::anyhow;
use api::{Api, WsResult};
use futures::FutureExt;
use gloo::{net::http::{Request}};
use yew::Callback;

use self::{auth::Auth, ws::WsConnection};

pub mod auth;
pub mod ws;

pub struct Connection {
    auth: Auth,
    ws: Rc<WsConnection>,
    connection_error_handler: Callback<ConnectionError>,
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.status() == other.status()
    }
}
impl Eq for Connection {}

#[derive(PartialEq, Eq)]
pub enum ConnectionStatus {
    LoggedOut,
    WsNotConnected,
    Connected,
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectionError {
    #[error("wrong or missing credentials in request")]
    Unauthorized,
    #[error("websocket connection closed or errored")]
    WsConnectionClose,
    #[error("HTTP request return error status: {0}")]
    HttpError(String),
    #[error(transparent)]
    OtherError(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ConnectionError>;

impl From<gloo::net::Error> for ConnectionError {
    fn from(err: gloo::net::Error) -> Self {
        ConnectionError::OtherError(anyhow!(err))
    }
}

impl ConnectionError {
    fn from_response(status_code: u16, text: String) -> Self {
        match status_code {
            401 => ConnectionError::Unauthorized,
            _ => ConnectionError::HttpError(format!("{}: {}", status_code, text)),
        }
    }
}

impl Connection {
    pub fn new(connection_error_handler: Callback<ConnectionError>) -> Self {
        Self {
            auth: Auth::default(),
            ws: Rc::new(WsConnection::default()),
            connection_error_handler,
        }
    }

    pub fn status(&self) -> ConnectionStatus {
        if !self.auth.authenticated() {
            ConnectionStatus::LoggedOut
        } else if !self.ws.connected() {
            ConnectionStatus::WsNotConnected
        } else {
            ConnectionStatus::Connected
        }
    }

    pub async fn login(&self, username: String, password: String) -> Result<()> {
        self.auth.login(username, password).await
    }

    pub fn logout(&self) {
        self.auth.logout()
    }

    pub fn connect_ws(
        &self,
        recv_callback: Callback<WsResult>,
    ) -> impl Future<Output = Result<()>> {
        if let Some(token) = self.auth.token() {
            self.ws
                .clone()
                .connect(token, recv_callback, self.connection_error_handler.clone())
                .left_future()
        } else {
            std::future::ready(Err(ConnectionError::Unauthorized)).right_future()
        }
    }

    pub fn disconnect_ws(&self) {
        self.ws.disconnect();
    }

    pub fn get_api<A: Api>(&self, url: &str) -> impl Future<Output = Result<<A as Api>::Response>> {
        if let Some(token) = self.auth.token() {
            let request = Request::get(url).header("Authorization", &format!("Bearer {}", token));
            fetch::<A>(request).left_future()
        } else {
            self.connection_error_handler
                .emit(ConnectionError::Unauthorized);
            std::future::ready(Err(ConnectionError::Unauthorized)).right_future()
        }
    }

    pub fn post_api<A: Api>(
        &self,
        url: &str,
        payload: <A as Api>::Request,
    ) -> impl Future<Output = Result<<A as Api>::Response>> {
        if let Some(token) = self.auth.token() {
            let url = url.to_owned();
            async move {
                let request = Request::post(&url)
                    .header("Authorization", &format!("Bearer {}", token))
                    .json(&payload)?;
                fetch::<A>(request).await
            }
            .left_future()
        } else {
            self.connection_error_handler
                .emit(ConnectionError::Unauthorized);
            std::future::ready(Err(ConnectionError::Unauthorized)).right_future()
        }
    }
}

async fn fetch<A: Api>(request: Request) -> Result<<A as Api>::Response> {
    Ok(request.send().await?.json::<<A as Api>::Response>().await?)
}
