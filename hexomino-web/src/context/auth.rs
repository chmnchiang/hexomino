use std::cell::RefCell;

use anyhow::{anyhow};
use api::{LoginRequest, LoginResponse};
use gloo::net::http::Request;

use crate::util::ResultExt;

use super::{Result, ConnectionError};

#[derive(Default)]
pub struct Auth {
    inner: RefCell<Option<AuthInner>>,
}

impl PartialEq for Auth {
    fn eq(&self, other: &Self) -> bool {
        self.authenticated() == other.authenticated()
    }
}
impl Eq for Auth {}

struct AuthInner {
    username: String,
    token: String,
}

impl Auth {
    pub fn authenticated(&self) -> bool {
        self.inner.borrow().is_some()
    }

    pub fn username(&self) -> Option<String> {
        Some(self.inner.borrow().as_ref()?.username.clone())
    }

    pub fn token(&self) -> Option<String> {
        Some(self.inner.borrow().as_ref()?.token.clone())
    }

    pub async fn login(&self, username: String, password: String) -> Result<()> {
        let payload = LoginRequest { username, password };
        let response = Request::post("/api/login").json(&payload)?.send().await?;
        if !response.ok() {
            let text = response.text().await.anyhow()?;
            return Err(ConnectionError::from_response(response.status(), text));
        }
        let response: LoginResponse = response.json().await?;
        let inner = AuthInner {
            username: response.username,
            token: response.token,
        };
        *self.inner.borrow_mut() = Some(inner);
        Ok(())
    }

    pub fn logout(&self) {
        *self.inner.borrow_mut() = None;
    }
}
