use api::{CreateRoomApi, ListRoomsApi, JoinRoomApi};
use gloo::timers::callback::Interval;
use itertools::Itertools;

use wasm_bindgen_futures::spawn_local;
use yew::{html, html::Scope, Component, Context, Html};

use crate::{context::ScopeExt, util::ResultExt};



pub struct RoomsView {
    rooms: Vec<api::Room>,
    _refresh_rooms_timer: Interval,
}

pub enum RoomsMsg {
    OnReceiveRooms(Vec<api::Room>),
    UpdateRooms,
}

const REFRESH_INTERVAL: u32 = 5_000;

impl Component for RoomsView {
    type Message = RoomsMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        Self::update_rooms(link.clone());
        let refresh_rooms_timer = Interval::new(REFRESH_INTERVAL, move || {
            Self::update_rooms(link.clone());
        });
        Self {
            rooms: vec![],
            _refresh_rooms_timer: refresh_rooms_timer,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use RoomsMsg::*;
        match msg {
            OnReceiveRooms(rooms) => {
                self.rooms = rooms;
                true
            }
            UpdateRooms => {
                Self::update_rooms(ctx.link().clone());
                false
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().clone();
        let room_to_html = {
            let link = link.clone();
            move |room: &api::Room| -> Html {
                let link = link.clone();
                let room_id = room.id;
                let users = room.users.iter().cloned().map(|user| user.name).join(", ");
                let join_callback = move |_| {
                    let link = link.clone();
                    spawn_local(async move {
                        let resp = link.connection().post_api::<JoinRoomApi>("/api/room/join", room_id)
                            .await;
                        let Ok(resp) = resp.log_err() else { return };
                        let Ok(()) = resp.log_err() else { return };
                        link.send_message(RoomsMsg::UpdateRooms);
                    })
                };
                let id_str = format!("{}", room_id.0);
                html! {
                    <tr>
                        <td style="vertical-align: middle; width: 20%">{id_str}</td>
                        <td style="vertical-align: middle">{users}</td>
                        <td style="text-align: right; width: 0%">
                            <button class="button is-success" onclick={join_callback}>{"Join"}</button>
                        </td>
                    </tr>
                }
            }
        };
        let onclick = move |_| {
            let link = link.clone();
            let connection = link.connection();
            spawn_local(async move {
                let resp = connection
                    .post_api::<CreateRoomApi>("/api/room/create", ())
                    .await;
                let Ok(resp) = resp.log_err().ignore_err() else { return };
                let Ok(_room_id) = resp.log_err().ignore_err() else { return };
                link.send_message(RoomsMsg::UpdateRooms);
            });
        };
        html! {
            <div class="columns is-centered">
                <div class="column is-half">
                    <div class="buttons">
                        <button class="button is-primary" {onclick}>
                            <span class="icon">
                                <i class="fa-solid fa-plus"></i>
                            </span>
                            <span> {"Create room"} </span>
                        </button>
                        <button class="button is-primary">
                            <span class="icon">
                                <i class="fa-solid fa-right-to-bracket"></i>
                            </span>
                            <span> {"Join a match room"} </span>
                        </button>
                        <button class="button" style="margin-left: auto;">
                            <span class="icon">
                                <i class="fa-solid fa-arrow-rotate-right"></i>
                            </span>
                        </button>
                    </div>
                    <h3 class="title">{"Public Rooms"}</h3>
                    <div class="field">
                        <p class="control has-icons-left">
                            <input class="input" placeholder="Search room by ID or user"/>
                            <span class="icon is-small is-left">
                                <i class="fa-solid fa-magnifying-glass"></i>
                            </span>
                        </p>
                    </div>
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{"Room ID"}</th>
                                <th>{"Users"}</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody> {
                            self.rooms.iter().map(room_to_html).collect::<Html>()
                        } </tbody>
                    </table>
                </div>
            </div>
        }
    }
}

impl RoomsView {
    fn update_rooms(link: Scope<Self>) {
        let connection = link.connection();
        let callback = link.callback(RoomsMsg::OnReceiveRooms);
        spawn_local(async move {
            let _resp = connection
                .get_api::<ListRoomsApi>("/api/rooms")
                .await
                .log_err()
                .map_cb(callback);
        });
    }
}
