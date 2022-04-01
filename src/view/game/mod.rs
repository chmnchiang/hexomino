use std::{cell::RefCell, rc::Rc};

use log::debug;
use yew::{html, Component, Context, Html, Properties};

use self::{
    end_view::EndView,
    pick_view::PickView,
    place_view::PlaceView,
    state::{GameState, GameViewState, SharedGameViewState},
};
use crate::game::{
    hexo::Hexo,
    state::{Action, GamePhase, Player},
};

mod board_canvas;
mod end_view;
mod hexo_svg;
mod hexo_table;
mod pick_view;
mod place_view;
mod state;
mod turn_indicator;

#[derive(PartialEq, Properties)]
pub struct GameProps;

pub struct GameView {
    state: SharedGameViewState,
}

fn fast_forward_to_place(state: &mut GameState) {
    for hexo in Hexo::all_hexos() {
        state.play(Action::Pick { hexo }).unwrap();
    }
}

impl Component for GameView {
    type Message = Action;
    type Properties = GameProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let mut game_state = GameState::new();
        Self {
            state: Rc::new(RefCell::new(GameViewState {
                game_state,
                me: Player::First,
            })),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let res = self.state.borrow_mut().game_state.play(msg);
        debug!("play = {res:?}");
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let game_state = &self.state.borrow().game_state;
        let send_pick = ctx.link().callback(|hexo| Action::Pick { hexo });
        let send_place = ctx
            .link()
            .callback(|moved_hexo| Action::Place { hexo: moved_hexo });
        html! {
            <div> {
                match game_state.phase() {
                    GamePhase::Pick => html!{
                        <PickView state={self.state.clone()} send_pick={send_pick}/>
                    },
                    GamePhase::Place => html!{
                        <PlaceView state={self.state.clone()} send_place={send_place}/>
                    },
                    GamePhase::End => html!{
                        <EndView state={self.state.clone()}/>
                    }
                }
            } </div>
        }
    }
}
