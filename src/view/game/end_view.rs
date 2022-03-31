use yew::{function_component, html, Properties};

use crate::game::state::Player;

use super::state::SharedGameViewState;

#[derive(PartialEq, Properties)]
pub struct EndViewProps {
    pub state: SharedGameViewState,
}

#[function_component(EndView)]
pub fn end_view(props: &EndViewProps) -> Html {
    let win_banner = match props.state.borrow().game_state.winner().unwrap() {
        Player::First => html! { <h1 class="title my-foreground">{ "Player 1 Wins" }</h1> },
        Player::Second => html! { <h1 class="title their-foreground">{ "Player 2 Wins" }</h1> },
    };
    html! {
        <div class="columns is-mobile is-centered">
            <div class="column is-narrow">
            { win_banner }
            </div>
        </div>
    }
}
