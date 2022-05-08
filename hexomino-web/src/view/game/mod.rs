use std::{cell::RefCell, rc::Rc};

use api::{GameAction, GameActionApi, GameActionRequest, WsResult, WsResponse, GameEvent};
use hexomino_core::{Action, GamePhase, Hexo};
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html, Properties};

use self::{end_view::EndView, pick_view::PickView, place_view::PlaceView};
use crate::{
    context::{ScopeExt, connection::ws::WsListenerToken},
    game::{new_game, CoreGameState, GameBundle, GameMode, GameState, SharedGameState}, util::ResultExt,
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
pub struct GameProps {
    //pub game_mode: GameMode,
}

pub struct GameView {
    //game_bundle: Option<GameBundle>,
    game_state: Option<SharedGameState>,
    _ws_listener_token: WsListenerToken,
}

pub enum GameMsg {
    StartGame,
    UserPlay(Action),
    OnUserPlay(Action),
    //StateChanged,
}

fn fast_forward_to_place(state: &mut CoreGameState) {
    for hexo in Hexo::all_hexos() {
        state.play(Action::Pick { hexo }).unwrap();
    }
}

impl Component for GameView {
    type Message = GameMsg;
    type Properties = GameProps;

    fn create(ctx: &Context<Self>) -> Self {
        let connection = ctx.link().connection();
        let ws_listener_token =
            connection.register_ws_callback(ctx.link().batch_callback(|resp: Rc<WsResult>| {
                log::debug!("{:?}", resp);
                match &*resp {
                    WsResponse::GameEvent(event) => {
                        if let GameEvent::UserPlay(action) = event {
                            Some(GameMsg::UserPlay(*action))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }));
        Self { game_state: None, _ws_listener_token: ws_listener_token }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: GameMsg) -> bool {
        use GameMsg::*;
        match msg {
            StartGame => {
                self.game_state = Some(Rc::new(RefCell::new(GameState::new(
                    "p1".to_string(),
                    "p2".to_string(),
                ))));
                true
            }
            UserPlay(action) => {
                let _ = self
                    .game_state
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .core_game_state
                    .play(action);
                true
            }
            OnUserPlay(action) => {
                let connection = ctx.link().connection();
                spawn_local(async move {
                    let _ = connection
                        .post_api::<GameActionApi>("/api/game/action", GameAction::Play(action))
                        .await.log_err();
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Some(ref game_state) = self.game_state else { return html!{} };
        let game_state_borrow = game_state.borrow();
        let core_state = &game_state_borrow.core_game_state;

        let send_pick = ctx
            .link()
            .callback(|hexo| GameMsg::OnUserPlay(Action::Pick { hexo }));
        let send_place = ctx
            .link()
            .callback(|moved_hexo| GameMsg::OnUserPlay(Action::Place { hexo: moved_hexo }));
        html! {
            <div> {
                match core_state.phase() {
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
            } </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if _first_render {
            ctx.link().send_message(GameMsg::StartGame);
        }
        //if self.game_bundle.is_none() {
            //ctx.link()
                //.send_message(GameMsg::StartGame(ctx.props().game_mode));
        //}
    }
}
