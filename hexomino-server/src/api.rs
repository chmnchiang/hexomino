use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    headers::{authorization::Bearer, Authorization},
    response::IntoResponse,
    routing::post,
    Extension, Json, Router, TypedHeader,
};
use chrono::{Duration, Utc};
use hexomino_api::{AuthPayload, AuthResponse};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::DbPool;

pub fn routes() -> Router {
    Router::new().route("/login", post(login_handler))
}

async fn login_handler(
    Json(payload): Json<AuthPayload>,
    Extension(db): Extension<DbPool>,
) -> Result<Json<AuthResponse>, AuthError> {
    let user = sqlx::query!(
        r#"
        SELECT id, name, password FROM Users
        WHERE name = ?
        "#,
        payload.username
    )
    .fetch_one(&db)
    .await
    .map_err(|_| AuthError)?;

    // TODO: Make this secure. We work on password hash instead. Yet, this is not really important
    // in our case where we generate all the password.
    if user.password != payload.password {
        return Err(AuthError);
    }

    let claims = Claims {
        exp: (Utc::now() + Duration::days(1)).timestamp(),
        id: user.id,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"hao123"),
    )
    .map_err(|_| AuthError)?;

    Ok(Json(AuthResponse {
        username: user.name,
        token,
    }))
}

pub struct AuthError;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    exp: i64,
    pub id: i64,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::UNAUTHORIZED, "Wrong or missing credentials").into_response()
    }
}

#[async_trait]
impl<B: Send> FromRequest<B> for Claims {
    type Rejection = AuthError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, AuthError> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| AuthError)?;

        authorize_jwt(bearer.token()).await
    }
}

pub async fn authorize_jwt(bearer: &str) -> Result<Claims, AuthError> {
    trace!("authorizing jwt");
    let data = jsonwebtoken::decode::<Claims>(
        bearer,
        &jsonwebtoken::DecodingKey::from_secret(b"hao123"),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| AuthError)?;
    Ok(data.claims)
}
