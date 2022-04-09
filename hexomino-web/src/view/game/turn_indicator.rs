use yew::{function_component, html, Properties};

use hexomino_core::Player;

#[derive(Properties, PartialEq)]
pub struct TurnIndicatorProps {
    pub current_player: Option<Player>,
    #[prop_or("Player 1".to_string())]
    pub player_1_name: String,
    #[prop_or("Player 2".to_string())]
    pub player_2_name: String,
}

#[function_component(TurnIndicator)]
pub fn turn_indicator(props: &TurnIndicatorProps) -> Html {
    const WIDTH: i32 = 3000;
    const HEIGHT: i32 = 100;
    const D_LEN: i32 = 50;
    const SPACE: i32 = 20;
    const MARGIN: i32 = 10;
    const FONT_SIZE: i32 = HEIGHT * 4 / 5;
    const FONT_PADDING: i32 = 20;
    let viewbox = format!(
        "{} {} {} {}",
        -MARGIN,
        -MARGIN,
        WIDTH + MARGIN * 2,
        HEIGHT + MARGIN * 2,
    );
    let shape_player_1 = format!(
        "M0 0 H{} l{} {} H{} Z",
        D_LEN + WIDTH / 2 - SPACE / 2,
        -D_LEN * 2,
        HEIGHT,
        0,
    );
    let shape_player_2 = format!(
        "M{} {} H{} l{} {} H{} Z",
        WIDTH,
        HEIGHT,
        -D_LEN + WIDTH / 2 + SPACE / 2,
        D_LEN * 2,
        -HEIGHT,
        WIDTH,
    );
    let (player1_opacity, player2_opacity) = match props.current_player {
        Some(Player::First) => (1.0, 0.5),
        Some(Player::Second) => (0.5, 1.0),
        None => (0.5, 0.5),
    };
    let player1_style = format!("fill: rgba(30, 180, 0, {})", player1_opacity);
    let player2_style = format!("fill: rgba(180, 30, 0, {})", player2_opacity);
    html! {
        <div style="width: 100%">
            <svg width="100%" viewBox={viewbox}>
            <path d={shape_player_1} style={player1_style}/>
            <path d={shape_player_2} style={player2_style}/>
            <text x={FONT_PADDING.to_string()} y={(HEIGHT/2).to_string()}
                font-size="80" alignment-baseline="central">{props.player_1_name.clone()}</text>
            <text x={(WIDTH - FONT_PADDING).to_string()} y={(HEIGHT/2).to_string()} font-size="80"
                text-anchor="end" alignment-baseline="central">{props.player_2_name.clone()}</text>
            </svg>
        </div>
    }
}
