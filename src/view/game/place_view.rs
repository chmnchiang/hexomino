use std::{cell::RefCell, rc::Rc};

use anyhow::Context as _;
use itertools::Itertools;
use log::info;
use piet::kurbo::Vec2;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{
    function_component, html, use_effect, use_effect_with_deps, use_mut_ref, use_node_ref,
    use_reducer, Callback, Component, Context, Html, NodeRef, Properties, Reducible,
};

use crate::{
    game::{
        constants,
        hexo::{Hexo, MovedHexo},
        state::Player,
    },
    render::Renderer,
    view::game::{hexo_table::{HexoTable, StyledHexo}, board_canvas::BoardCanvas},
};

use super::{
    turn_indicator::TurnIndicator, state::SharedGameViewState,
};

#[derive(Properties, PartialEq)]
pub struct PlaceViewProps {
    pub state: SharedGameViewState,
    pub send_place: Callback<MovedHexo>,
}

#[derive(Debug, Default)]
struct PlaceViewState {
    pub selected_hexo: Option<Hexo>,
}

pub enum PlaceAction {
    Select(Hexo),
    Placed(MovedHexo),
}

#[derive(Default)]
pub struct PlaceView {
    renderer: Option<Renderer>,
    state: PlaceViewState,
}

impl Component for PlaceView {
    type Message = PlaceAction;
    type Properties = PlaceViewProps;

    fn create(ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use PlaceAction::*;
        match msg {
            Select(hexo) => {
                self.state.selected_hexo = Some(hexo);
            }
            Placed(moved_hexo) => {
                ctx.props().send_place.emit(moved_hexo);
                self.state.selected_hexo = None;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = ctx.props().state.borrow();
        let game_state = &state.game_state;
        let current_player = game_state.current_player().unwrap();
        let select_onclick = ctx.link().callback(|hexo| PlaceAction::Select(hexo));
        let place_hexo_callback =
            ctx.link().callback(|hexo| PlaceAction::Placed(hexo));

        let me = state.me;
        let player_hexos = game_state.inventory().hexos_of(current_player).iter();

        let styled_hexos = if let Some(selected_hexo) = self.state.selected_hexo {
            player_hexos
                .map(|hexo| {
                    (
                        hexo,
                        match (hexo == selected_hexo, current_player) {
                            (false, _) => None,
                            (true, Player::First) => Some("my-picked-hexo".to_string()),
                            (true, Player::Second) => Some("their-picked-hexo".to_string()),
                        },
                    )
                })
                .collect::<Vec<StyledHexo>>()
        } else {
            player_hexos
                .map(|hexo| (hexo, None))
                .collect::<Vec<StyledHexo>>()
        };

        html! {
            <>
                <TurnIndicator current_player={game_state.current_player()}/>
                <BoardCanvas state={ctx.props().state.clone()} selected_hexo={self.state.selected_hexo} {place_hexo_callback}/>
                <HexoTable {styled_hexos} on_hexo_click={select_onclick}/>
            </>
        }
    }
}
