use std::{rc::Rc, time::Duration, str::FromStr};

use api::{
    JoinedRoom, LeaveRoomApi, MatchConfig, RoomActionApi, RoomActionRequest, RoomId, RoomUser,
    WsResponse, WsResult, MatchSettings,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlSelectElement};
use yew::{html, Component, Context, Html};

use crate::{
    context::{connection::ws::WsListenerToken, ScopeExt},
    util::ResultExt, view::common::match_token_html,
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
                match_token: None,
                users: vec![],
                settings: MatchSettings {
                    config: MatchConfig::Normal,
                    number_of_games: 0,
                    play_time_limit: Duration::from_secs(0),
                },
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

        let config_onchange = {
            let connection = ctx.link().connection();
            move |event: Event| {
                let target = event.target().expect("Input event does not have a target");
                let select: HtmlSelectElement = target.unchecked_into();
                let Ok(config) = MatchConfig::from_str(&select.value()) else { return; };
                let connection = connection.clone();
                spawn_local(async move {
                    let _ = connection
                        .post_api::<RoomActionApi>(
                            "/api/room/action",
                            RoomActionRequest::SetConfig(config),
                        )
                        .await
                        .log_err();
                });
            }
        };

        fn user_to_html(user: &RoomUser) -> Html {
            html! {
                <tr>
                    <td>
                        <span class="icon"><i class="fa-solid fa-user"></i></span>
                        { user.user.name.clone() }
                    </td>
                    <td style="text-align: center; width: 25%; min-width: 100px;"> {
                        if user.is_ready {
                            html! { <span class="tag is-success">{"Ready"}</span> }
                        } else {
                            html! { <span class="tag is-warning">{"Not Ready"}</span> }
                        }
                    } </td>
                </tr>
            }
        }

        let room_title = format!("Room #{}", self.room.id);
        let number_of_games = format!("{}", self.room.settings.number_of_games);
        let play_time_limit = format!("{}s", self.room.settings.play_time_limit.as_secs());
        let config_select = html! {
            <select onchange={config_onchange}> {
                [(MatchConfig::Normal, "Normal Game"),
                 (MatchConfig::KnockoutStage, "Knockout Stage"),
                 (MatchConfig::ChampionshipStage, "Championship Stage")]
                    .into_iter()
                    .map(|(cf, display_name)| {
                        let value = format!("{}", cf);
                        let selected = cf == self.room.settings.config;
                        html! { <option {value} {selected}>{display_name}</option> }
                    })
                    .collect::<Html>()
            } </select>
        };
        html! {
            <div>
                <div class="columns is-centered">
                    <div class="column is-half">
                        <h2 class="title"> {room_title} </h2>
                        <p class="subtitle"> { match_token_html(&self.room.match_token, true) } </p>
                        <hr/>
                        <h2 class="title is-4" style="margin-bottom: 1rem">{"Users"}</h2>
                        <table class="table is-fullwidth is-hoverable is-bordered">
                            <tbody> {
                                self.room.users.iter().map(user_to_html).collect::<Html>()
                            } </tbody>
                        </table>
                        <hr/>
                        <h2 class="title is-4" style="margin-bottom: 1rem">{"Game Settings"}</h2>
                        <div class="columns">
                            <div class="column is-half">
                                <div class="field">
                                    <label class="label">{"Config"}</label>
                                    <div class="control">
                                        <div class="select is-fullwidth"> { config_select } </div>
                                    </div>
                                </div>
                            </div>
                            <div class="column is-half">
                                <ul style="list-style-type: disc; list-style-position: inside;">
                                    <li> <b style="margin-right: 10px;">{"Number of games:"}</b> {number_of_games} </li>
                                    <li> <b style="margin-right: 10px;">{"Time limit:"}</b> {play_time_limit} </li>
                                </ul>
                            </div>
                        </div>
                        <hr/>
                        if !self.self_is_ready(ctx) {
                            <button class="button is-medium is-fullwidth is-success" onclick={on_ready_click}>{"Ready"}</button>
                        } else {
                            <button class="button is-medium is-fullwidth is-warning" onclick={on_undo_ready_click}>{"Undo Ready"}</button>
                        }
                        <button class="button is-medium is-fullwidth is-danger"
                            style="margin-top: 1rem;" onclick={on_leave_click}>
                            <span class="icon"><i class="fa-solid fa-arrow-right-from-bracket"></i></span>
                            <span>{"Leave room"}</span>
                        </button>
                    </div>
                </div>
                <div class="columns is-centered">
                </div>
                <div class="columns is-centered">
                    <div class="column is-half">
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
