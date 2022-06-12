
use yew::{classes, function_component, html, Callback, Properties};

use hexomino_core::{Hexo};

use crate::view::common::hexo_svg::HexoSvg;

#[derive(Properties, PartialEq)]
pub struct HexoBlockProps {
    pub hexo: Hexo,
    #[prop_or(None)]
    pub style: Option<String>,
    #[prop_or_else(Callback::noop)]
    pub onclick: Callback<Hexo>,
}

#[function_component(HexoBlock)]
pub fn hexo_block(props: &HexoBlockProps) -> Html {
    let hexo = props.hexo;
    let onclick = props.onclick.reform(move |_| hexo);
    html! {
        <div style="width: 100%; height: 100%; border: 2px black solid"
            class={classes![&props.style, "hexo-div"]} {onclick}>
            <HexoSvg {hexo}/>
        </div>
    }
}
