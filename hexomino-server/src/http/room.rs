use api::{Api, CreateRoomApi, GetRoomApi, JoinRoomApi, JoinRoomRequest, ListRoomsApi};
use axum::Json;

use crate::kernel::{user::User, Kernel};

use super::{into_infallible_json_response, into_json_response, JsonResponse};

pub async fn list_rooms_handler() -> JsonResponse<<ListRoomsApi as Api>::Response> {
    into_infallible_json_response(Kernel::get().list_rooms().await)
}

pub async fn get_room_handler(
    user: User,
    Json(room_id): Json<<GetRoomApi as Api>::Request>,
) -> JsonResponse<<GetRoomApi as Api>::Response> {
    into_json_response(Kernel::get().get_room(user, room_id).await)
}

pub async fn create_room_handler(user: User) -> JsonResponse<<CreateRoomApi as Api>::Response> {
    into_json_response(Kernel::get().create_room(user).await)
}

pub async fn join_room_handler(
    user: User,
    Json(room_id): Json<JoinRoomRequest>,
) -> JsonResponse<<JoinRoomApi as Api>::Response> {
    into_json_response(Kernel::get().join_room(user, room_id).await)
}
