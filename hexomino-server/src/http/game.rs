use api::{Api, GameActionApi};
use axum::Json;

use crate::kernel::{user::User, Kernel};

use super::{into_json_response, JsonResponse};

pub async fn game_action_handler(
    user: User,
    Json(game_action): Json<<GameActionApi as Api>::Request>,
) -> JsonResponse<<GameActionApi as Api>::Response> {
    into_json_response(Kernel::get().game_action(user, game_action).await)
}
