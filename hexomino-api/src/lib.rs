mod auth;
mod room;
mod user;
mod ws;
mod game;

pub use auth::*;
pub use room::*;
pub use user::*;
pub use ws::*;
pub use game::*;

macro_rules! derive_api_data {
    () => {};
    ($item:item $($rest:item)*) => {
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        $item

        derive_api_data!($($rest)*);
    };
}
pub(crate) use derive_api_data;

pub trait ApiData:
    serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + Clone
{
}
impl<T> ApiData for T where
    T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + Clone
{
}

pub trait Api {
    type Request: ApiData;
    type Response: ApiData;
}
derive_api_data! {
    pub enum Never {}
}
