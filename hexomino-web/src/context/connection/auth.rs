use std::{cell::RefCell, rc::Rc};

use api::{LoginRequest, LoginResponse, User, RefreshTokenResponse, AuthResponse};
use gloo::{
    net::http::Request,
    storage::{errors::StorageError, LocalStorage, Storage},
};

use crate::util::ResultExt;

use super::{ConnectionError, Result};

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

#[derive(Clone, Debug)]
struct AuthInner {
    token: String,
    me: Rc<User>,
}

impl Auth {
    pub(super) fn authenticated(&self) -> bool {
        self.inner.borrow().is_some()
    }

    pub(super) fn me(&self) -> Option<Rc<User>> {
        Some(self.inner.borrow().as_ref()?.me.clone())
    }

    pub(super) fn token(&self) -> Option<String> {
        Some(self.inner.borrow().as_ref()?.token.clone())
    }

    pub(super) async fn login(&self, username: String, password: String) -> Result<()> {
        let payload = LoginRequest { username, password };
        let response = Request::post("/api/auth/login").json(&payload)?.send().await?;
        if !response.ok() {
            let text = response.text().await.anyhow()?;
            return Err(ConnectionError::from_response(response.status(), text));
        }
        let response: LoginResponse = response.json().await?;
        self.process_auth_response(response);
        Ok(())
    }

    pub(super) async fn load_and_refresh_token(&self) -> Result<()> {
        let token = load_token().ok_or(ConnectionError::Unauthorized)?;
        let request = Request::post("/api/auth/refresh_token")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&())?;
        let result = request.send().await?;
        if !result.ok() {
            return Err(ConnectionError::Unauthorized)
        }
        let response = result.json::<RefreshTokenResponse>().await?;
        self.process_auth_response(response);
        Ok(())
    }


    pub(super) fn logout(&self) {
        *self.inner.borrow_mut() = None;
        clear_token();
    }

    fn process_auth_response(&self, resp: AuthResponse) {
        save_token(&resp.token);
        let inner = AuthInner {
            me: Rc::new(resp.me),
            token: resp.token,
        };
        *self.inner.borrow_mut() = Some(inner);
    }
}

const TOKEN_SAVE_KEY: &str = "auth_token";

fn save_token(token: &str) {
    if let Err(err) = LocalStorage::set(TOKEN_SAVE_KEY, token) {
        log::error!("failed to store auth context: {}", err);
    }
}

fn load_token() -> Option<String> {
        match LocalStorage::get(TOKEN_SAVE_KEY) {
            Ok(token) => {
                Some(token)
            }
            Err(err) => {
                match err {
                    StorageError::KeyNotFound(_) => {
                        log::info!("auth context not found in storage");
                    }
                    _ => {
                        log::error!("failed to load auth context: {}", err);
                        clear_token();
                    }
                }
                None
            }
        }
}

fn clear_token() {
    LocalStorage::delete(TOKEN_SAVE_KEY);
}
