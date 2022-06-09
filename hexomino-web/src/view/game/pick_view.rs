use super::{bottom_message::BottomMessage};
use crate::{game::SharedGameState, view::game::hexo_table::HexoTable};
use hexomino_core::{Hexo};
use yew::{html, Callback, Component, Context, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct PickViewProps {
    pub state: SharedGameState,
    pub send_pick: Callback<Hexo>,
}

pub struct PickView;

impl Component for PickView {
    type Message = ();
    type Properties = PickViewProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = ctx.props().state.borrow();

        let me = ctx.props().state.borrow().me();
        let my_turn = state.core().current_player() == Some(me);
        let hexo_style_func = |hexo| {
            ctx.props()
                .state
                .borrow()
                .core()
                .inventory()
                .owner_of(hexo)
                .map(|player| {
                    if player == me {
                        "my-picked-hexo"
                    } else {
                        "their-picked-hexo"
                    }
                    .to_string()
                })
        };
        let styled_hexos = Hexo::all_hexos()
            .map(|hexo| (hexo, hexo_style_func(hexo)))
            .collect::<Vec<_>>();
        html! {
            <div class="block">
                <HexoTable {styled_hexos} on_hexo_click={ctx.props().send_pick.clone()}/>
                <BottomMessage> {
                    if my_turn {
                        html! {
                            <p style="font-size: 1.5rem">
                                <b> {"Your turn: "} </b>
                                <span> { "Pick a hexomino by clicking on its block" } </span>
                            </p>
                        }
                    } else {
                        html!{
                            <p style="font-size: 1.5rem">
                                <b> {"Opponent's turn: "} </b>
                                <span> { "Wait for your opponent to pick a hexomino" } </span>
                            </p>
                        }
                    }
                } </BottomMessage>
            </div>
        }
    }
}
