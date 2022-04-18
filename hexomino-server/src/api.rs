use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    headers::{authorization::Bearer, Authorization},
    response::IntoResponse,
    routing::{get, post},
    Json, Router, TypedHeader,
};
use chrono::{Duration, Utc};
use hexomino_api::{AuthPayload, AuthResponse};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::trace;

pub fn routes() -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/protect", get(protect_handler))
}

async fn login_handler(Json(payload): Json<AuthPayload>) -> Result<Json<AuthResponse>, AuthError> {
    if payload.username != "hao123" || payload.password != "hao123" {
        return Err(AuthError);
    }

    let claims = Claims {
        exp: (Utc::now() + Duration::days(1)).timestamp(),
        name: payload.username,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"hao123"),
    )
    .map_err(|_| AuthError)?;

    Ok(Json(AuthResponse {
        username: claims.name,
        token,
    }))
}

async fn protect_handler(claims: Claims) -> String {
    format!("You are {}", claims.name)
}

struct AuthError;

#[derive(Serialize, Deserialize)]
struct Claims {
    exp: i64,
    name: String,
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

        authorize_jwt(bearer).await
    }
}

async fn authorize_jwt(bearer: Bearer) -> Result<Claims, AuthError> {
    trace!("authorizing jwt");
    let data = jsonwebtoken::decode::<Claims>(
        bearer.token(),
        &jsonwebtoken::DecodingKey::from_secret(b"hao123"),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| AuthError)?;
    Ok(data.claims)
}
