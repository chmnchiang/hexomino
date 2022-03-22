use super::{
    hexo_svg::{self, HexoSvg},
    state::SharedGameViewState,
    turn_indicator::TurnIndicator,
};
use crate::{
    game::{hexo::Hexo, state::Player},
    view::game::hexo_table::HexoTable,
};
use yew::{html, Callback, Component, Context, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct PickViewProps {
    pub state: SharedGameViewState,
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
        let game_state = &state.game_state;

        let hexo_style_func = |hexo| {
            ctx.props()
                .state
                .borrow()
                .game_state
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
                <TurnIndicator current_player={game_state.current_player()}/>
                <HexoTable {styled_hexos} on_hexo_click={ctx.props().send_pick.clone()}/>
            </>
        }
    }
}
