use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::trace;
use uuid::Uuid;

#[cfg(feature = "competition-mode")]
#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    exp: i64,
    pub id: Uuid,
}

#[cfg(feature = "competition-mode")]
pub async fn create_jwt_token(id: Uuid) -> Option<String> {
    let claims = Claims {
        exp: (Utc::now() + Duration::days(1)).timestamp(),
        id,
    };
    encode(
        &Header::default(),
        &claims,
        // TODO: This is not safe, should read the secret from env.
        &EncodingKey::from_secret(b"hao123"),
    )
    .ok()
}

#[cfg(feature = "competition-mode")]
pub async fn authorize_jwt(bearer: &str) -> Option<Claims> {
    trace!("authorizing jwt");
    decode::<Claims>(
        bearer,
        // TODO: This is not safe, should read the secret from env.
        &DecodingKey::from_secret(b"hao123"),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .ok()
}

#[cfg(not(feature = "competition-mode"))]
#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    exp: i64,
    pub id: Uuid,
    pub username: String,
}

#[cfg(not(feature = "competition-mode"))]
pub async fn create_jwt_token(id: Uuid, username: String) -> Option<String> {
    let claims = Claims {
        exp: (Utc::now() + Duration::days(1)).timestamp(),
        id,
        username,
    };
    encode(
        &Header::default(),
        &claims,
        // TODO: This is not safe, should read the secret from env.
        &EncodingKey::from_secret(b"hao123"),
    )
    .ok()
}

#[cfg(not(feature = "competition-mode"))]
pub async fn authorize_jwt(bearer: &str) -> Option<Claims> {
    trace!("authorizing jwt");
    decode::<Claims>(
        bearer,
        // TODO: This is not safe, should read the secret from env.
        &DecodingKey::from_secret(b"hao123"),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .ok()
}
