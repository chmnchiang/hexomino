use api::{RoomError, Never};


use serde::Serialize;

pub type ApiResult<T, E> = std::result::Result<T, Error<E>>;

pub enum Error<E: ApiError> {
    Api(E),
    Common(CommonError),
}

pub trait ApiError: Serialize {}
impl ApiError for Never {}
impl ApiError for RoomError {}

#[derive(thiserror::Error, Debug)]
pub enum CommonError {
    #[error("wrong or missing credentials in request")]
    Unauthorized,
    #[cfg(feature = "internal-debug")]
    #[error("internal error: {0}")]
    Internal(anyhow::Error),
    #[cfg(not(feature = "internal-debug"))]
    #[error("internal error")]
    Internal(anyhow::Error),
}

impl<E: ApiError> From<E> for Error<E> {
    fn from(err: E) -> Self {
        Error::Api(err)
    }
}

impl<E: ApiError> From<anyhow::Error> for Error<E> {
    fn from(err: anyhow::Error) -> Self {
        Error::Common(CommonError::Internal(err))
    }
}
