use api::{CreateRoomApi, ListRoomsApi};
use gloo::timers::callback::Interval;
use itertools::Itertools;
use log::debug;
use wasm_bindgen_futures::spawn_local;
use yew::{html, Callback, Component, Context, Html};

use crate::util::ResultExt;

use super::MainContext;

pub struct RoomsView {
    rooms: Vec<api::Room>,
    _refresh_rooms_timer: Interval,
}

pub enum RoomsMsg {
    UpdateRooms(Vec<api::Room>),
}

const REFRESH_INTERVAL: u32 = 5_000;

impl Component for RoomsView {
    type Message = RoomsMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _) = ctx.link().context::<MainContext>(Callback::noop()).unwrap();
        let callback = ctx.link().callback(RoomsMsg::UpdateRooms);
        let refresh_rooms_timer = Interval::new(REFRESH_INTERVAL, move || {
            let context = context.clone();
            let callback = callback.clone();
            spawn_local(async move {
                let _resp = context
                    .connection
                    .get_api::<ListRoomsApi>("/api/rooms")
                    .await
                    .log_err()
                    .map_cb(callback);
            });
        });
        Self {
            rooms: vec![],
            _refresh_rooms_timer: refresh_rooms_timer,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        use RoomsMsg::*;
        match msg {
            UpdateRooms(rooms) => {
                self.rooms = rooms;
                true
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (context, _) = ctx.link().context::<MainContext>(Callback::noop()).unwrap();
        fn room_to_html(room: &api::Room) -> Html {
            let id = format!("{}", room.id.0);
            let users = room.users.iter().cloned().map(|user| user.name).join(", ");
            html! {
                <tr>
                    <td>{id}</td>
                    <td>{users}</td>
                </tr>
            }
        }
        let onclick = move |_| {
            let context = context.clone();
            spawn_local(async move {
                let resp = context
                    .connection
                    .post_api::<CreateRoomApi>("/api/room/create", ())
                    .await;
                let Ok(resp) = resp.log_err().ignore_err() else { return };
                let Ok(room_id) = resp.log_err().ignore_err() else { return };
                debug!("room_id = {room_id}")
            });
        };
        html! {
            <>
                <table class="table">
                    <thead>
                        <tr>
                            <th>{"Room ID"}</th>
                            <th>{"Users"}</th>
                        </tr>
                    </thead>
                    <tbody> {
                        self.rooms.iter().map(room_to_html).collect::<Html>()
                    } </tbody>
                </table>
                <button class="button is-success" {onclick}>{"Create room"}</button>
            </>
        }
    }
}
