use std::{future::Future};

use anyhow::anyhow;
use api::{Api};
use futures::FutureExt;
use gloo::net::http::Request;
use yew::{Callback};

use self::{auth::Auth, ws::{WsConnection, WsCallback, WsListenerToken}};

pub mod auth;
pub mod ws;

pub struct Connection {
    auth: Auth,
    ws: WsConnection,
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
            auth: Auth::try_load(),
            ws: WsConnection::default(),
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

    pub async fn connect_ws(&self) -> Result<()> {
        if let Some(token) = self.auth.token() {
            self.ws
                .connect(token, self.connection_error_handler.clone())
                .await
        } else {
            Err(ConnectionError::Unauthorized)
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

    pub fn register_ws_callback(&self, callback: WsCallback) -> WsListenerToken {
        self.ws.register_callback(callback)
    }
}

async fn fetch<A: Api>(request: Request) -> Result<<A as Api>::Response> {
    let result = request.send().await?;
    if !result.ok() {
        match result.status() {
            401 => return Err(ConnectionError::Unauthorized),
            _ => {
                let mut err_msg = result.status_text();
                if let Ok(s) = result.text().await && !s.is_empty() {
                    err_msg.push_str(&s);
                }
                return Err(ConnectionError::HttpError(result.status_text()));
            }
        }
    }

    Ok(result.json::<<A as Api>::Response>().await?)
}
