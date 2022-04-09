use hexomino_core::Player;
use yew::{classes, function_component, html, Properties};

use crate::game::SharedGameState;

#[derive(PartialEq, Properties)]
pub struct EndViewProps {
    pub state: SharedGameState,
}

#[function_component(EndView)]
pub fn end_view(props: &EndViewProps) -> Html {
    let state = props.state.borrow();
    let winner = state.core_game_state.winner().unwrap();
    let style = match winner {
        Player::First => "my-foreground",
        Player::Second => "their-foreground",
    };
    html! {
        <div class="columns is-mobile is-centered">
            <div class="column is-narrow">
                <h1 class={classes!("title", style)}>{ format!("{} Wins", state.name_of(winner)) }</h1>
            </div>
        </div>
    }
}
