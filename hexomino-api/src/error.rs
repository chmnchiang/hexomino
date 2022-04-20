use std::fmt;

use crate::Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Client(msg) => write!(f, "Client error: {}", msg),
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Client(msg) => write!(f, "Client error: {}", msg),
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        if cfg!(feature = "internal-debug") {
            let msg = format!("{err:?}");
            Error::Internal(msg)
        } else {
            Error::Internal("something went wrong...".into())
        }
    }
}

#[macro_export]
macro_rules! cerr {
    ($msg:literal $(,)?) => {
        $crate::Error::Client(format!($msg))
    };
    ($err:expr $(,)?) => {
        $crate::Error::Client(format!("{}", $err))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::Error::Client($format!($fmt, $($arg)*))
    };
}
