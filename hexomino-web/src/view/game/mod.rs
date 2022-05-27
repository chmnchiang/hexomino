use std::{cell::RefCell, rc::Rc};

use api::{
    GameEvent, MatchAction, MatchActionApi, MatchInfo, SyncMatchApi, UserPlay, WsResponse, WsResult, GameStartInfo, GameEndInfo,
};
use hexomino_core::{Action, GamePhase, Hexo};
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html, Properties};

use self::{end_view::EndView, pick_view::PickView, place_view::PlaceView};
use crate::{
    context::{connection::ws::WsListenerToken, ScopeExt},
    game::{GameState, SharedGameState, MatchState},
    util::ResultExt, view::game::turn_indicator::TurnIndicator,
};

mod board_canvas;
mod board_renderer;
mod end_view;
mod hexo_svg;
mod hexo_table;
mod pick_view;
mod place_view;
mod turn_indicator;

#[derive(PartialEq, Properties)]
pub struct GameProps {}

pub struct GameView {
    mtch: Option<MatchState>,
    game_status: GameStatus,
    _ws_listener_token: WsListenerToken,
}

pub enum GameMsg {
    OnSyncMatch(api::MatchState),
    OnStartGame(GameStartInfo),
    OnUserPlay(UserPlay),
    OnGameEnd(GameEndInfo),
    UserPlay(Action),
}

#[derive(PartialEq, Eq)]
pub enum GameStatus {
    NotStarted,
    Playing,
    Ended,
}

impl Component for GameView {
    type Message = GameMsg;
    type Properties = GameProps;

    fn create(ctx: &Context<Self>) -> Self {
        let connection = ctx.link().connection();
        let ws_listener_token =
            connection.register_ws_callback(ctx.link().batch_callback(|resp: Rc<WsResult>| {
                match (&*resp).clone() {
                    WsResponse::GameEvent(GameEvent::GameStart(info)) => {
                        Some(GameMsg::OnStartGame(info))
                    }
                    WsResponse::GameEvent(GameEvent::UserPlay(action)) => {
                        Some(GameMsg::OnUserPlay(action))
                    }
                    WsResponse::GameEvent(GameEvent::GameEnd(info)) => {
                        //info.reason
                        Some(GameMsg::OnUserPlay(action))
                    }
                    _ => None,
                }
            }));
        Self {
            mtch: None,
            game_status: GameStatus::NotStarted,
            _ws_listener_token: ws_listener_token,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: GameMsg) -> bool {
        use GameMsg::*;
        match msg {
            OnSyncMatch(match_state) => {
                self.sync_match(match_state);
                return true
            }
            OnStartGame(info) => {
                self.on_start_game(info, ctx);
                if let Some(mtch) = &mut self.mtch && self.game_status == GameStatus::NotStarted {
                    mtch.new_game(info.you);
                    true
                } else {
                    self.do_sync_match(ctx);
                    false
                }
            }
            OnUserPlay(action) => {
                if let Some(mtch) = &mut self.mtch && mtch.update(action.action).is_ok() {
                    true
                } else {
                    self.do_sync_match(ctx);
                    false
                }
            }
            UserPlay(action) => {
                let connection = ctx.link().connection();
                spawn_local(async move {
                    let _ = connection
                        .post_api::<MatchActionApi>("/api/game/action", MatchAction::Play(action))
                        .await
                        .log_err();
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        fn loader_html() -> Html {
            log::debug!("loader html");
            html! {
                <div class="pageloader is-active">
                    <span class="title">{"Waiting for the game to start..."}</span>
                </div>
            }
        }
        let Some(mtch) = &self.mtch else { return loader_html(); };
        let Some(game_state) = &mtch.game() else { return loader_html() };

        let send_pick = ctx
            .link()
            .callback(|hexo| GameMsg::UserPlay(Action::Pick { hexo }));
        let send_place = ctx
            .link()
            .callback(|moved_hexo| GameMsg::UserPlay(Action::Place { hexo: moved_hexo }));
        html! {
            <div>
            <TurnIndicator/>
            {
                match game_state.borrow().core().phase() {
                    GamePhase::Pick => html!{
                        <PickView state={game_state.clone()} send_pick={send_pick}/>
                    },
                    GamePhase::Place => html!{
                        <PlaceView state={game_state.clone()} send_place={send_place}
                        is_locked={false}/>
                    },
                    GamePhase::End => html!{
                        <EndView state={game_state.clone()}/>
                    }
                }
            }
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.do_sync_match(ctx);
        }
    }
}

impl GameView {
    fn sync_match(&mut self, mtch: api::MatchState) {
        self.mtch = Some(MatchState::from_api(mtch));
    }

    fn do_sync_match(&self, ctx: &Context<Self>) {
        let callback = ctx.link().callback(GameMsg::OnSyncMatch);
        let connection = ctx.link().connection();
        spawn_local(async move {
            let resp = connection
                .post_api::<SyncMatchApi>("/api/game/sync", ())
                .await;
            let Ok(result) = resp.log_err() else { return };
            let Ok(match_state) = result.log_err() else { return };
            callback.emit(match_state);
        })
    }
}
