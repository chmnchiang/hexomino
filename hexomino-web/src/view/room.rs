use std::rc::Rc;

use api::{JoinedRoom, RoomActionApi, RoomActionRequest, RoomId, RoomUser, WsResponse, WsResult, LeaveRoomApi};
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html};

use crate::{
    context::{connection::ws::WsListenerToken, ScopeExt},
    util::ResultExt,
};

pub struct RoomView {
    room: JoinedRoom,
    _ws_listener_token: WsListenerToken,
}

pub enum RoomMsg {
    UpdateRoom(JoinedRoom),
}

impl Component for RoomView {
    type Message = RoomMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let connection = ctx.link().connection();
        let callback = ctx.link().callback(RoomMsg::UpdateRoom);
        let ws_listener_token =
            connection.register_ws_callback(ctx.link().batch_callback(|resp: Rc<WsResult>| {
                match &*resp {
                    WsResponse::RoomUpdate(room) => Some(RoomMsg::UpdateRoom(room.clone())),
                    _ => None,
                }
            }));

        spawn_local(async move {
            if let Ok(result) = connection.post_api::<api::GetRoomApi>("/api/room", ()).await
                && let Ok(result) = result.log_err() {
                    callback.emit(result);
                }
        });

        Self {
            room: JoinedRoom {
                id: RoomId(0),
                users: vec![],
            },
            _ws_listener_token: ws_listener_token,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RoomMsg::UpdateRoom(room) => {
                self.room = room;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_ready_click = {
            let connection = ctx.link().connection();
            move |_| {
                let connection = connection.clone();
                spawn_local(async move {
                    let _ = connection
                        .post_api::<RoomActionApi>("/api/room/action", RoomActionRequest::Ready)
                        .await
                        .log_err();
                });
            }
        };

        let on_undo_ready_click = {
            let connection = ctx.link().connection();
            move |_| {
                let connection = connection.clone();
                spawn_local(async move {
                    let _ = connection
                        .post_api::<RoomActionApi>("/api/room/action", RoomActionRequest::UndoReady)
                        .await
                        .log_err();
                });
            }
        };

        let on_leave_click = {
            let connection = ctx.link().connection();
            move |_| {
                let connection = connection.clone();
                spawn_local(async move {
                    let _ = connection
                        .post_api::<LeaveRoomApi>("/api/room/leave", ())
                        .await
                        .log_err();
                });
            }
        };

        fn user_to_html(user: &RoomUser) -> Html {
            html! {
                <tr>
                    <td> {
                        user.user.name.clone()
                    } </td>
                    <td style="text-align: right;"> {
                        if user.is_ready {
                            html! { <span class="tag is-success">{"Ready"}</span> }
                        } else {
                            html! {}
                        }
                    } </td>
                </tr>
            }
        }

        let room_title = format!("Room #{}", self.room.id);

        html! {
            <div>
                <div class="columns is-centered">
                    <div class="column is-half">
                        <h2 class="title">{room_title}</h2>
                        <table class="table is-fullwidth is-hoverable">
                            <thead>
                                <tr>
                                    <th>{"Users"}</th>
                                    <th style="width: 25%;"></th>
                                </tr>
                            </thead>
                            <tbody> {
                                self.room.users.iter().map(user_to_html).collect::<Html>()
                            } </tbody>
                        </table>
                    </div>
                </div>
                <div class="columns is-centered">
                    if !self.self_is_ready(ctx) {
                        <div class="column is-half">
                            <button class="button is-medium is-fullwidth is-success" onclick={on_ready_click}>{"Ready"}</button>
                        </div>
                    } else {
                        <div class="column is-half">
                            <button class="button is-medium is-fullwidth is-warning" onclick={on_undo_ready_click}>{"Undo Ready"}</button>
                        </div>
                    }
                </div>
                <div class="columns is-centered">
                    <div class="column is-half">
                        <button class="button is-medium is-fullwidth is-warning" onclick={on_leave_click}>
                            <span class="icon"><i class="fa-solid fa-arrow-right-from-bracket"></i></span>
                            <span>{"Leave room"}</span>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}

impl RoomView {
    fn self_is_ready(&self, ctx: &Context<Self>) -> bool {
        let connection = ctx.link().connection();
        let id = connection.me().unwrap().id;
        let room_me = self.room.users.iter().find(|u| u.user.id == id);
        room_me.is_some_and(|u| u.is_ready)
    }
}
