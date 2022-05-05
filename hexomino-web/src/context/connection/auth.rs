use std::cell::RefCell;

use api::{LoginRequest, LoginResponse};
use gloo::{
    net::http::Request,
    storage::{errors::StorageError, LocalStorage, Storage},
};
use serde::{Deserialize, Serialize};

use crate::util::ResultExt;

use super::{ConnectionError, Result};

pub struct Auth {
    inner: RefCell<Option<AuthInner>>,
}

impl PartialEq for Auth {
    fn eq(&self, other: &Self) -> bool {
        self.authenticated() == other.authenticated()
    }
}
impl Eq for Auth {}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AuthInner {
    username: String,
    token: String,
}

impl Auth {
    pub fn try_load() -> Self {
        Self {
            inner: RefCell::new(AuthInner::load()),
        }
    }

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
        inner.save();
        *self.inner.borrow_mut() = Some(inner);
        Ok(())
    }

    pub fn logout(&self) {
        *self.inner.borrow_mut() = None;
    }
}

const AUTH_SAVE_KEY: &str = "auth";

impl AuthInner {
    fn save(&self) {
        if let Err(err) = LocalStorage::set(AUTH_SAVE_KEY, self.clone()) {
            log::error!("failed to store auth context: {}", err);
        }
    }

    fn load() -> Option<AuthInner> {
        log::debug!("create");
        match LocalStorage::get(AUTH_SAVE_KEY) {
            Ok(auth) => {
                Some(auth)
            }
            Err(err) => {
                match err {
                    StorageError::KeyNotFound(_) => {
                        log::info!("auth context not found in storage");
                    }
                    _ => {
                        log::error!("failed to load auth context: {}", err);
                    }
                }
                None
            }
        }
    }
}
