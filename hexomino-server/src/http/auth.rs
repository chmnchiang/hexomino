use api::{LoginRequest, LoginResponse};
use axum::{Extension, Json};

use crate::http::JsonResponse;
use crate::result::CommonError;
use crate::{auth::create_jwt_token, DbPool};

pub async fn login_handler(
    Json(request): Json<LoginRequest>,
    Extension(db): Extension<DbPool>,
) -> JsonResponse<LoginResponse> {
    let user = sqlx::query!(
        r#"
        SELECT id, name, password FROM Users
        WHERE name = $1
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
        username: user.name,
        token: create_jwt_token(user.id)
            .await
            .ok_or_else(|| CommonError::Unauthorized)?,
    }))
}
