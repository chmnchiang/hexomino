use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::trace;

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    exp: i64,
    pub id: Uuid,
}

pub async fn create_jwt_token(id: Uuid) -> Option<String> {
    let claims = Claims {
        exp: (Utc::now() + Duration::days(1)).timestamp(),
        id,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"hao123"),
    )
    .ok()
}

pub async fn authorize_jwt(bearer: &str) -> Option<Claims> {
    trace!("authorizing jwt");
    decode::<Claims>(
        bearer,
        &DecodingKey::from_secret(b"hao123"),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .ok()
}
