use api::{ApiData, Never};
use axum::{
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use hyper::StatusCode;

use crate::result::{ApiError, ApiResult, CommonError, Error};

use self::{
    auth::login_handler,
    room::{create_room_handler, join_room_handler, list_rooms_handler, get_room_handler},
};

mod auth;
mod room;

pub fn routes() -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/rooms", get(list_rooms_handler))
        .route("/room", post(get_room_handler))
        .route("/room/create", post(create_room_handler))
        .route("/room/join", post(join_room_handler))
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

pub fn into_infallible_json_response<T: ApiData>(
    result: ApiResult<T, Never>,
) -> JsonResponse<T> {
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
        (self.status_code(), format!("{self}")).into_response()
    }
}
