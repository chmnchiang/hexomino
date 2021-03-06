use api::{ApiData, Never};
use axum::{
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use hyper::StatusCode;

use crate::result::{ApiError, ApiResult, CommonError, Error};

use self::{
    auth::{login_handler, refresh_token_handler},
    game::{match_action_handler, sync_match_handler},
    match_history::list_user_match_histories_handler,
    room::{
        create_room_handler, get_room_handler, join_room_handler, leave_room_handler,
        list_rooms_handler, room_action_handler, create_or_join_match_room_handler,
    },
};

mod auth;
mod game;
mod match_history;
mod room;

pub fn routes() -> Router {
    Router::new()
        .route("/auth/login", post(login_handler))
        .route("/auth/refresh_token", post(refresh_token_handler))
        .route("/rooms", get(list_rooms_handler))
        .route("/room", post(get_room_handler))
        .route("/room/create", post(create_room_handler))
        .route("/room/join", post(join_room_handler))
        .route("/room/join_match", post(create_or_join_match_room_handler))
        .route("/room/leave", post(leave_room_handler))
        .route("/room/action", post(room_action_handler))
        .route("/game/sync", post(sync_match_handler))
        .route("/game/action", post(match_action_handler))
        .route(
            "/match_history/user_list",
            get(list_user_match_histories_handler),
        )
}

pub type JsonResponse<T> = std::result::Result<Json<T>, CommonError>;

pub fn into_json_response<T: ApiData, E: ApiError>(
    result: ApiResult<T, E>,
) -> JsonResponse<Result<T, E>> {
    match result {
        Ok(x) => Ok(Json(Ok(x))),
        Err(Error::Api(x)) => Ok(Json(Err(x))),
        Err(Error::Common(x)) => Err(x),
    }
}

pub fn into_infallible_json_response<T: ApiData>(result: ApiResult<T, Never>) -> JsonResponse<T> {
    match result {
        Ok(x) => Ok(Json(x)),
        Err(Error::Api(x)) => match x {},
        Err(Error::Common(x)) => Err(x),
    }
}

impl CommonError {
    fn status_code(&self) -> StatusCode {
        match self {
            CommonError::Unauthorized => StatusCode::UNAUTHORIZED,
            CommonError::Internal(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for CommonError {
    fn into_response(self) -> Response {
        tracing::debug!("{:?}", &self);
        (self.status_code(), format!("{self}")).into_response()
    }
}
