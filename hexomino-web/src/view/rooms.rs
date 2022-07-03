use api::{CreateOrJoinMatchRoomApi, CreateRoomApi, JoinRoomApi, ListRoomsApi, MatchToken};
use gloo::timers::callback::Interval;
use itertools::Itertools;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, InputEvent};
use yew::{classes, html, html::Scope, Component, Context, Html, NodeRef};

use crate::{context::ScopeExt, util::ResultExt};

use super::common::match_token_html;

pub struct RoomsView {
    fetched_rooms: Vec<api::Room>,
    filter: String,
    modal_is_opened: bool,
    match_token_input_ref: NodeRef,
    _refresh_rooms_timer: Interval,
}

pub enum RoomsMsg {
    OnReceiveRooms(Vec<api::Room>),
    UpdateRooms,
    OpenModal,
    CloseModal,
    JoinMatchRoom,
    UpdateFilter(String),
}

const REFRESH_INTERVAL: u32 = 10_000;

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
            fetched_rooms: vec![],
            filter: String::new(),
            modal_is_opened: false,
            match_token_input_ref: NodeRef::default(),
            _refresh_rooms_timer: refresh_rooms_timer,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use RoomsMsg::*;
        match msg {
            OnReceiveRooms(rooms) => {
                self.fetched_rooms = rooms;
                true
            }
            UpdateRooms => {
                Self::update_rooms(ctx.link().clone());
                false
            }
            UpdateFilter(text) => {
                self.filter = text;
                true
            }
            OpenModal => {
                self.modal_is_opened = true;
                true
            }
            CloseModal => {
                self.modal_is_opened = false;
                let input = self
                    .match_token_input_ref
                    .cast::<HtmlInputElement>()
                    .expect("can't cast the ref to input element");
                input.set_value("");
                true
            }
            JoinMatchRoom => {
                let input = self
                    .match_token_input_ref
                    .cast::<HtmlInputElement>()
                    .expect("can't cast the ref to input element");
                let match_token = input.value();
                let context = ctx.link().main_context();
                let connection = context.connection();
                spawn_local(async move {
                    let Ok(result) = connection
                        .post_api::<CreateOrJoinMatchRoomApi>(
                            "/api/room/join_match",
                            MatchToken(match_token),
                        )
                        .await
                        .show_err(&context) else { return; };
                    let _ = result.show_err(&context);
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().clone();
        let create_room_onclick = {
            let link = link.clone();
            move |_| {
                let link = link.clone();
                let connection = link.connection();
                spawn_local(async move {
                    let resp = connection
                        .post_api::<CreateRoomApi>("/api/room/create", ())
                        .await;
                    let Ok(resp) = resp.log_err().ignore_err() else { return };
                    let Ok(_room_id) = resp.log_err().ignore_err() else { return };
                });
            }
        };
        let join_match_onclick = ctx.link().callback(|_| RoomsMsg::OpenModal);

        let refresh_onclick = link.callback(|_| RoomsMsg::UpdateRooms);
        let search_oninput = link.callback(|event: InputEvent| {
            let target = event.target().expect("Input event does not have a target");
            let input_element: HtmlInputElement = target.unchecked_into();
            RoomsMsg::UpdateFilter(input_element.value())
        });

        let room_to_html = {
            let context = link.main_context();
            move |room: &api::Room| -> Html {
                let context = context.clone();
                let room_id = room.id;
                let users = room.users.iter().cloned().map(|user| user.name).join(", ");
                let join_callback = move |_| {
                    let context = context.clone();
                    spawn_local(async move {
                        let resp = context
                            .connection()
                            .post_api::<JoinRoomApi>("/api/room/join", room_id)
                            .await;
                        let Ok(resp) = resp.show_err(&context) else { return };
                        let _ = resp.show_err(&context);
                    })
                };
                let id_str = format!("{}", room_id.0);
                let player_cnt_str = format!("{}/2", room.users.len());
                html! {
                    <tr>
                        <td style="vertical-align: middle;">{id_str}</td>
                        <td style="vertical-align: middle;">{
                            match_token_html(&room.match_token, false)
                        }</td>
                        <td style="vertical-align: middle">{users}</td>
                        <td style="vertical-align: middle;">
                            <span class="icon">
                                <i class="fa-solid fa-user"></i>
                            </span>
                            <span> { player_cnt_str } </span>
                        </td>
                        <td style="text-align: right;">
                            <button class="button is-success" style={(room.users.len() == 2).then_some("visibility: hidden")}
                                onclick={join_callback}>{"Join"}</button>
                        </td>
                    </tr>
                }
            }
        };
        let filter_room = |&room: &&api::Room| {
            let filter_str = &self.filter;
            if filter_str.is_empty() {
                return true;
            }
            if room.id.to_string().contains(filter_str) {
                return true;
            }
            if room.match_token.is_some_and(|t| t.0.contains(filter_str)) {
                return true;
            }
            if room.users.iter().any(|user| user.name.contains(filter_str)) {
                return true;
            }
            false
        };
        let mut rooms = self.fetched_rooms.iter().filter(filter_room).collect_vec();
        rooms.sort_by(|r1, r2| {
            let r1_users_cnt = r1.users.len();
            let r2_users_cnt = r2.users.len();
            if r1_users_cnt != r2_users_cnt {
                return r1_users_cnt.cmp(&r2_users_cnt);
            }
            r1.id.cmp(&r2.id).reverse()
        });

        html! {
            <>
                <div class="columns is-centered">
                    <div class="column is-two-thirds">
                        <div class="buttons">
                            <button class="button is-primary" onclick={create_room_onclick}>
                                <span class="icon">
                                    <i class="fa-solid fa-plus"></i>
                                </span>
                                <span> {"Create room"} </span>
                            </button>
                            <button class="button is-primary" onclick={join_match_onclick}>
                                <span class="icon">
                                    <i class="fa-solid fa-right-to-bracket"></i>
                                </span>
                                <span> {"Join a match room"} </span>
                            </button>
                            <button class="button" style="margin-left: auto;" onclick={refresh_onclick}>
                                <span class="icon">
                                    <i class="fa-solid fa-arrow-rotate-right"></i>
                                </span>
                                <span> {"Refresh rooms"} </span>
                            </button>
                        </div>
                        <h3 class="title">{"Public Rooms"}</h3>
                        <div class="field">
                            <p class="control has-icons-left">
                                <input class="input" placeholder="Search room by ID, match token, or user"
                                  oninput={search_oninput}/>
                                <span class="icon is-small is-left">
                                    <i class="fa-solid fa-magnifying-glass"></i>
                                </span>
                            </p>
                        </div>
                        <table class="table is-fullwidth is-hoverable">
                            <thead>
                                <tr>
                                    <th style="width: 15%">{"Room ID"}</th>
                                    <th style="width: 25%; min-width: 180px;">{"Match type"}</th>
                                    <th>{"Users"}</th>
                                    <th style="width: 80px"></th>
                                    <th style="width: 80px"></th>
                                </tr>
                            </thead>
                            <tbody> {
                                rooms.into_iter()
                                    .map(room_to_html)
                                    .collect::<Html>()
                            } </tbody>
                        </table>
                    </div>
                </div>
                { self.join_match_room_modal(ctx) }
            </>
        }
    }
}

impl RoomsView {
    fn join_match_room_modal(&self, context: &Context<Self>) -> Html {
        let join_onclick = context.link().callback(|_| RoomsMsg::JoinMatchRoom);
        let cancel_onclick = context.link().callback(|_| RoomsMsg::CloseModal);
        html! {
            <div class={classes!("modal", self.modal_is_opened.then_some("is-active"))}>
                <div class="modal-background"></div>
                <div class="modal-content">
                    <div class="box">
                        <h3 class="subtitle">{"Join a match room"}</h3>
                        <div style="display: flex; justify-content: space-between">
                            <div class="field has-addons">
                                <div class="control">
                                    <input class="input" type="text" placeholder="Match room token"
                                        ref={self.match_token_input_ref.clone()}/>
                                </div>
                                <div class="control">
                                    <button class="button is-info" onclick={join_onclick}>{"Join match"}</button>
                                </div>
                            </div>
                            <div class="control" style="margin-left: 20px">
                                <button class="button is-danger" onclick={cancel_onclick}>
                                    <span class="icon"><i class="fa-solid fa-xmark"></i></span>
                                    <span>{"Cancel"}</span>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    }

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
