use api::{Api, LoginRequest, LoginResponse, RefreshTokenApi, RefreshTokenResponse, UserId};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::{Extension, Json, TypedHeader};
use uuid::Uuid;

use crate::auth::authorize_jwt;
use crate::http::JsonResponse;

use crate::auth::create_jwt_token;
use crate::kernel::user::unwrap_name_or_unnamed;
use crate::result::CommonError;

#[cfg(feature = "competition-mode")]
use crate::DbPool;

#[cfg(feature = "competition-mode")]
pub async fn login_handler(
    Json(request): Json<LoginRequest>,
    Extension(db): Extension<DbPool>,
) -> JsonResponse<LoginResponse> {
    let user = sqlx::query!(
        r#"
        SELECT id, name, password FROM Users
        WHERE username = $1
        "#,
        request.username
    )
    .fetch_one(&db)
    .await
    .map_err(|_| CommonError::Unauthorized)?;

    // TODO: Make this secure. We work on password hash instead. Yet, this is not really important
    // in our case where we generate all the password.
    if user.password != request.password {
        return Err(CommonError::Unauthorized);
    }

    Ok(Json(LoginResponse {
        me: api::User {
            id: UserId(user.id),
            name: unwrap_name_or_unnamed(user.name),
        },
        token: create_jwt_token(user.id)
            .await
            .ok_or(CommonError::Unauthorized)?,
    }))
}

#[cfg(not(feature = "competition-mode"))]
pub async fn login_handler(Json(request): Json<LoginRequest>) -> JsonResponse<LoginResponse> {
    if request.username.is_empty() || request.username.len() > 10 {
        return Err(CommonError::Unauthorized);
    }
    let id = Uuid::new_v4();
    let token = create_jwt_token(id, request.username.clone())
        .await
        .ok_or(CommonError::Unauthorized)?;
    Ok(Json(LoginResponse {
        me: api::User {
            id: UserId(id),
            name: request.username,
        },
        token,
    }))
}

#[cfg(feature = "competition-mode")]
pub async fn refresh_token_handler(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Extension(db): Extension<DbPool>,
) -> JsonResponse<<RefreshTokenApi as Api>::Response> {
    let claim = authorize_jwt(bearer.token())
        .await
        .ok_or(CommonError::Unauthorized)?;
    let user = sqlx::query!(
        r#"
        SELECT id, name FROM Users
        WHERE id = $1
        "#,
        claim.id,
    )
    .fetch_one(&db)
    .await
    .map_err(|_| CommonError::Unauthorized)?;

    Ok(Json(RefreshTokenResponse {
        me: api::User {
            id: UserId(user.id),
            name: user.name.unwrap_or_else(|| "<Unnamed>".to_string()),
        },
        token: create_jwt_token(user.id)
            .await
            .ok_or(CommonError::Unauthorized)?,
    }))
}

#[cfg(not(feature = "competition-mode"))]
pub async fn refresh_token_handler(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> JsonResponse<<RefreshTokenApi as Api>::Response> {
    let claim = authorize_jwt(bearer.token())
        .await
        .ok_or(CommonError::Unauthorized)?;

    Ok(Json(RefreshTokenResponse {
        me: api::User {
            id: UserId(claim.id),
            name: claim.username.clone(),
        },
        token: create_jwt_token(claim.id, claim.username)
            .await
            .ok_or(CommonError::Unauthorized)?,
    }))
}
