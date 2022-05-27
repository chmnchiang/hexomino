use api::{Api, MatchActionApi, SyncMatchApi};
use axum::Json;

use crate::kernel::{user::User, Kernel};

use super::{into_json_response, JsonResponse};

pub async fn sync_match_handler(
    user: User,
) -> JsonResponse<<SyncMatchApi as Api>::Response> {
    into_json_response(Kernel::get().sync_match(user).await)
}

pub async fn match_action_handler(
    user: User,
    Json(match_action): Json<<MatchActionApi as Api>::Request>,
) -> JsonResponse<<MatchActionApi as Api>::Response> {
    into_json_response(Kernel::get().match_action(user, match_action).await)
}
