use crate::{derive_api_data, Api, User};

derive_api_data! {
    pub struct LoginRequest {
        pub username: String,
        pub password: String,
    }
    pub struct AuthResponse {
        pub token: String,
        pub me: User,
    }
}

pub type LoginResponse = AuthResponse;
pub type RefreshTokenRequest = ();
pub type RefreshTokenResponse = AuthResponse;

pub struct LoginApi;
impl Api for LoginApi {
    type Request = LoginRequest;
    type Response = LoginResponse;
}

pub struct RefreshTokenApi;
impl Api for RefreshTokenApi {
    type Request = RefreshTokenRequest;
    type Response = RefreshTokenResponse;
}
