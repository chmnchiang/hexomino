

use api::Never;

pub trait ResultNeverExt<T> {
    fn sure(self) -> T;
}

impl<T> ResultNeverExt<T> for Result<T, Never> {
    fn sure(self) -> T {
        match self {
            Ok(x) => x,
            Err(err) => match err {},
        }
    }
}
