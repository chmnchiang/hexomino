use super::turn_indicator::TurnIndicator;
use crate::{game::SharedGameState, view::game::hexo_table::HexoTable};
use hexomino_core::{Hexo, Player};
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
        //let core_state = &state.core_game_state;

        let hexo_style_func = |hexo| {
            ctx.props()
                .state
                .borrow()
                .core()
                .inventory()
                .owner_of(hexo)
                .map(|player| match player {
                    Player::First => "my-picked-hexo".to_string(),
                    Player::Second => "their-picked-hexo".to_string(),
                })
        };
        let styled_hexos = Hexo::all_hexos()
            .map(|hexo| (hexo, hexo_style_func(hexo)))
            .collect::<Vec<_>>();
        html! {
            <>
                //<TurnIndicator current_player={core_state.current_player()}
                    //player_1_name={state.name_of(Player::First).to_string()}
                    //player_2_name={state.name_of(Player::Second).to_string()}/>
                <HexoTable {styled_hexos} on_hexo_click={ctx.props().send_pick.clone()}/>
            </>
        }
    }
}
