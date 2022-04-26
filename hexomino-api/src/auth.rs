use crate::{derive_api_data, Api};

pub struct LoginApi;
impl Api for LoginApi {
    type Request = LoginRequest;
    type Response = LoginResponse;
}
derive_api_data! {
    pub struct LoginRequest {
        pub username: String,
        pub password: String,
    }
    pub struct LoginResponse {
        pub username: String,
        pub token: String,
    }
}
