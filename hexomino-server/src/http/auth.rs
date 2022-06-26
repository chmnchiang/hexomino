use api::{Api, LoginRequest, LoginResponse, RefreshTokenApi, RefreshTokenResponse, UserId};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::{Extension, Json, TypedHeader};

use crate::auth::authorize_jwt;
use crate::http::JsonResponse;

use crate::kernel::user::unwrap_name_or_unnamed;
use crate::result::CommonError;
use crate::{auth::create_jwt_token, DbPool};

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
