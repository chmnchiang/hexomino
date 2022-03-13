use yew::{function_component, html, Properties};

use crate::game::state::Player;

#[derive(Properties, PartialEq)]
pub struct TurnIndicatorProps {
    pub current_player: Option<Player>,
}

#[function_component(TurnIndicator)]
pub fn turn_indicator(props: &TurnIndicatorProps) -> Html {
    html! {
        <p>
        {
            match props.current_player {
                Some(Player::First) => "<First Player>",
                Some(Player::Second) => "<Second Player>",
                None => "<No Player>",
            }
        }
        </p>
    }
}
