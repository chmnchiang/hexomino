use guard::guard;
use hexomino_core::{Action, GamePhase, Hexo};
use yew::{html, Component, Context, Html, Properties};

use self::{end_view::EndView, pick_view::PickView, place_view::PlaceView};
use crate::game::{new_game, CoreGameState, GameBundle, GameMode};

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
    pub game_mode: GameMode,
}

pub struct GameView {
    game_bundle: Option<GameBundle>,
}

pub enum GameMsg {
    StartGame(GameMode),
    UserPlay(Action),
    StateChanged,
}

fn fast_forward_to_place(state: &mut CoreGameState) {
    for hexo in Hexo::all_hexos() {
        state.play(Action::Pick { hexo }).unwrap();
    }
}

impl Component for GameView {
    type Message = GameMsg;
    type Properties = GameProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { game_bundle: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: GameMsg) -> bool {
        use GameMsg::*;
        match msg {
            StartGame(game_mode) => {
                self.game_bundle = Some(new_game(
                    game_mode,
                    ctx.link().callback(|()| GameMsg::StateChanged),
                ));
                true
            }
            UserPlay(action) => {
                let _ = self
                    .game_bundle
                    .as_ref()
                    .unwrap()
                    .game
                    .clone()
                    .user_play(action);
                false
            }
            StateChanged => true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        guard!(let Some(ref game_bundle) = self.game_bundle else { return html!{} });
        let game_state = &game_bundle.game_state;
        let game_state_borrow = game_state.borrow();
        let core_state = &game_state_borrow.core_game_state;
        let send_pick = ctx
            .link()
            .callback(|hexo| GameMsg::UserPlay(Action::Pick { hexo }));
        let send_place = ctx
            .link()
            .callback(|moved_hexo| GameMsg::UserPlay(Action::Place { hexo: moved_hexo }));
        html! {
            <div> {
                match core_state.phase() {
                    GamePhase::Pick => html!{
                        <PickView state={game_state.clone()} send_pick={send_pick}/>
                    },
                    GamePhase::Place => html!{
                        <PlaceView state={game_state.clone()} send_place={send_place}
                            is_locked={!game_bundle.game.user_can_play()}/>
                    },
                    GamePhase::End => html!{
                        <EndView state={game_state.clone()}/>
                    }
                }
            } </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if self.game_bundle.is_none() {
            ctx.link()
                .send_message(GameMsg::StartGame(ctx.props().game_mode));
        }
    }
}
