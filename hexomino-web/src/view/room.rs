use std::rc::Rc;

use api::{JoinedRoom, RoomActionApi, RoomActionRequest, RoomId, WsResponse, WsResult};
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html, Properties, classes};

use crate::{
    context::{connection::ws::WsListenerToken, ScopeExt},
    util::ResultExt,
};

pub struct RoomView {
    room: JoinedRoom,
    _ws_listener_token: WsListenerToken,
}

#[derive(Properties, PartialEq)]
pub struct RoomProps {
    pub room_id: RoomId,
}

pub enum RoomMsg {
    UpdateRoom(JoinedRoom),
}

impl Component for RoomView {
    type Message = RoomMsg;
    type Properties = RoomProps;

    fn create(ctx: &Context<Self>) -> Self {
        let room_id = ctx.props().room_id;
        let connection = ctx.link().connection();
        let callback = ctx.link().callback(RoomMsg::UpdateRoom);
        let ws_listener_token =
            connection.register_ws_callback(ctx.link().batch_callback(|resp: Rc<WsResult>| {
                log::debug!("{:?}", resp);
                match &*resp {
                    WsResponse::RoomUpdate(room) => Some(RoomMsg::UpdateRoom(room.clone())),
                    _ => None,
                }
            }));

        spawn_local(async move {
            if let Ok(result) = connection.post_api::<api::GetRoomApi>("/api/room", room_id).await
                && let Ok(result) = result.log_err() {
                    callback.emit(result);
                }
        });

        Self {
            room: JoinedRoom {
                id: ctx.props().room_id,
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
        html! {
            <>
                <div class="columns is-centered">
                    <div class="column is-one-quarter">
                        {self.user_html_card(0)}
                    </div>
                    <div class="column is-one-quarter">
                        {self.user_html_card(1)}
                    </div>
                </div>
                <div class="columns is-centered">
                    <div class="column is-half">
                        <button class="button is-medium is-fullwidth is-success" onclick={on_ready_click}>{"Ready"}</button>
                    </div>
                </div>
            </>
        }
    }
}

impl RoomView {
    fn user_html_card(&self, index: usize) -> Html {
        let user = self
            .room
            .users
            .get(index);
        let card_color = user.and_then(|user| user.is_ready.then(|| "is-success"));
        let inner = user
            .map(|user| {
                html! {
                    <p>{user.user.name.clone()}</p>
                }
            })
            .unwrap_or_else(|| html!());
        html! {
            <div class={classes!("card", card_color)}>
                <div class="card-content" style="min-height: 200px">
                { inner }
                </div>
            </div>
        }
    }
}
