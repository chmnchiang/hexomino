use yew::{function_component, html, Properties};

use hexomino_core::Player;

#[derive(Properties, PartialEq)]
pub struct TurnIndicatorProps {
    #[prop_or(Player::First)]
    pub me: Player,
    pub current_player: Option<Player>,
    #[prop_or([0, 0])]
    pub scores: [u32; 2],
    #[prop_or(["Player 1".to_string(), "Player 2".to_string()])]
    pub player_names: [String; 2],
}

pub fn player_color_style(is_me: bool, opacity: f64) -> String {
    if is_me {
        format!("fill: rgba(30, 180, 0, {})", opacity)
    } else {
        format!("fill: rgba(180, 30, 0, {})", opacity)
    }
}

#[function_component(TurnIndicator)]
pub fn turn_indicator(props: &TurnIndicatorProps) -> Html {
    const WIDTH: i32 = 3000;
    const HEIGHT: i32 = 100;
    const SCORE_LEN: i32 = 150;
    const D_LEN: i32 = 50;
    const SPACE: i32 = 20;
    const MARGIN: i32 = 10;
    const FONT_SIZE: i32 = HEIGHT * 4 / 5;
    const FONT_PADDING: i32 = 20;

    let viewbox = format!("{} {} {} {}", 0, 0, WIDTH, HEIGHT);
    let shape_score_1 = format!("M0 0 h{} v{} h{} Z", SCORE_LEN, HEIGHT, -SCORE_LEN);
    let shape_player_1 = format!(
        "M{} 0 H{} l{} {} H{} Z",
        SCORE_LEN + MARGIN,
        D_LEN + WIDTH / 2 - SPACE / 2,
        -D_LEN * 2,
        HEIGHT,
        SCORE_LEN + MARGIN,
    );
    let shape_score_2 = format!(
        "M{} {} h{} v{} h{} Z",
        WIDTH, 0, -SCORE_LEN, HEIGHT, SCORE_LEN
    );
    let shape_player_2 = format!(
        "M{} {} H{} l{} {} H{} Z",
        WIDTH - SCORE_LEN - MARGIN,
        HEIGHT,
        -D_LEN + WIDTH / 2 + SPACE / 2,
        D_LEN * 2,
        -HEIGHT,
        WIDTH - SCORE_LEN - MARGIN,
    );
    let (player1_opacity, player2_opacity) = match props.current_player {
        Some(Player::First) => (1.0, 0.5),
        Some(Player::Second) => (0.5, 1.0),
        None => (0.5, 0.5),
    };
    let player1_style = player_color_style(props.me == Player::First, player1_opacity);
    let player2_style = player_color_style(props.me == Player::Second, player2_opacity);
    html! {
        <div style="width: 100%;">
            <svg width="100%" style="min-height: 30px;" viewBox={viewbox}>
            <path d={shape_score_1} style={player1_style.clone()}/>
            <path d={shape_player_1} style={player1_style}/>
            <path d={shape_score_2} style={player2_style.clone()}/>
            <path d={shape_player_2} style={player2_style}/>
            <text x={(SCORE_LEN/2).to_string()} y={(HEIGHT/2).to_string()}
                font-size="80" alignment-baseline="central" text-anchor="middle">{props.scores[0]}</text>
            <text x={(FONT_PADDING + SCORE_LEN + MARGIN).to_string()} y={(HEIGHT/2).to_string()}
                font-size="80" alignment-baseline="central">{props.player_names[0].clone()}</text>
            <text x={(WIDTH - SCORE_LEN/2).to_string()} y={(HEIGHT/2).to_string()}
                font-size="80" alignment-baseline="central" text-anchor="middle">{props.scores[1]}</text>
            <text x={(WIDTH - FONT_PADDING - SCORE_LEN - MARGIN).to_string()} y={(HEIGHT/2).to_string()} font-size="80"
                text-anchor="end" alignment-baseline="central">{props.player_names[1].clone()}</text>
            </svg>
        </div>
    }
}
