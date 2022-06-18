use api::{Api, ListUserMatchHistoriesApi};

use crate::kernel::{user::User, Kernel};

use super::{JsonResponse, into_infallible_json_response};

pub async fn list_user_match_histories_handler(
    user: User,
) -> JsonResponse<<ListUserMatchHistoriesApi as Api>::Response> {
    into_infallible_json_response(Kernel::get().list_user_match_histories(user).await)
}
