use piet::kurbo::Vec2;
use yew::{function_component, html, Html, Properties};

use hexomino_core::{Hexo, Pos};

const BLOCK_LEN: f64 = 20.0;

#[derive(Properties, PartialEq, Eq)]
pub struct HexoSvgProps {
    pub hexo: Hexo,
}

fn center_of_mass(hexo: Hexo) -> Vec2 {
    let mut res = Vec2::ZERO;
    for point in hexo.tiles().map(Vec2::from) {
        res += point + Vec2::new(0.5, 0.5);
    }
    res / 6.0
}

#[function_component(HexoSvg)]
pub fn hexo_svg(props: &HexoSvgProps) -> Html {
    let hexo = props.hexo;
    let center = center_of_mass(hexo);

    let tile_to_html = |pos: Pos| {
        let diff_pos = Vec2::from(pos) - center;
        let x = diff_pos.x * 20.0;
        let y = diff_pos.y * 20.0;
        html! {
            <rect x={x.to_string()} y={y.to_string()} width="20" height="20"
                style="stroke: #606060; stroke-width: 2px; fill: none;"/>
        }
    };

    let border_to_html = |(pos1, pos2): (Pos, Pos)| {
        let diff_pos1 = Vec2::from(pos1) - center;
        let diff_pos2 = Vec2::from(pos2) - center;
        let x1 = diff_pos1.x * BLOCK_LEN;
        let y1 = diff_pos1.y * BLOCK_LEN;
        let x2 = diff_pos2.x * BLOCK_LEN;
        let y2 = diff_pos2.y * BLOCK_LEN;
        html! {
            <line x1={x1.to_string()} y1={y1.to_string()}
                x2={x2.to_string()} y2={y2.to_string()}
                style="stroke: black; stroke-width: 3px;"/>
        }
    };

    html! {
        <svg class="has-ratio" viewBox="-65 -65 130 130">
            { hexo.tiles().map(tile_to_html).collect::<Html>() }
            { hexo.borders().map(border_to_html).collect::<Html>() }
        </svg>
    }
}
