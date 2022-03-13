use piet::kurbo::Vec2;
use yew::{function_component, html, Html, Properties};

use crate::game::{hexo::Hexo, pos::Pos};

#[derive(Properties, PartialEq)]
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

fn build_hexo_svg(hexo: Hexo) -> Html {
    fn tile_to_html(pos: Pos, center: Vec2) -> Html {
        let diff_pos = Vec2::from(pos) - center;
        let x = diff_pos.x * 20.0;
        let y = diff_pos.y * 20.0;
        html! {
            <rect x={x.to_string()} y={y.to_string()} width="20" height="20"
                style="stroke:black; stroke-width:3px; fill:none;"/>
        }
    }

    fn tiles_to_html(hexo: Hexo) -> Html {
        let center = center_of_mass(hexo);
        hexo.tiles()
            .map(|pos| tile_to_html(pos, center))
            .collect::<Html>()
    }

    html! {
        <svg class="has-ratio" viewBox="-65 -65 130 130">
            { tiles_to_html(hexo) }
        </svg>
    }
}

#[function_component(HexoSvg)]
pub fn hexo_svg(props: &HexoSvgProps) -> Html {
    html! {
        <div class="card">
            <div class="card-image">
                <figure class="image is-1by1">
                    { build_hexo_svg(props.hexo) }
                </figure>
            </div>
        </div>
    }
}
