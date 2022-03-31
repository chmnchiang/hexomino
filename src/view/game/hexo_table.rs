use std::iter::repeat;

use itertools::Itertools;
use yew::{function_component, html, Callback, Html, Properties};

use super::hexo_svg::HexoSvg;
use crate::game::hexo::Hexo;

pub type StyledHexo = (Hexo, Option<String>);

#[derive(PartialEq, Properties)]
pub struct HexoTableProps {
    pub styled_hexos: Vec<StyledHexo>,
    #[prop_or_default]
    pub on_hexo_click: Callback<Hexo>,
}

#[function_component(HexoTable)]
pub fn hexo_table(props: &HexoTableProps) -> Html {
    const CHUNK_SIZE: usize = 9;

    fn hexo_chunk_html<'a>(
        chunk: impl Iterator<Item = &'a StyledHexo>,
        onclick: Callback<Hexo>,
    ) -> Html {
        let chunk = chunk.map(Some).chain(repeat(None)).take(CHUNK_SIZE);

        chunk.map(|hexo| html!{
            <div class="square-block hexo-block"> {
                    match hexo {
                        Some((hexo, style)) => html!{
                            <HexoSvg hexo={*hexo} style={style.clone()} onclick={onclick.clone()}/>
                        },
                        None => html!{},
                    }
            } </div>
        }).collect::<Html>()
    }

    html! {
        <div style="width: 100%;">
            <div class="hexo-table-flexbox" style="width: 100%;"> {
                props.styled_hexos
                    .iter()
                    .chunks(CHUNK_SIZE)
                    .into_iter()
                    .map(|chunk| hexo_chunk_html(chunk, props.on_hexo_click.clone()))
                    .collect::<Html>()
            } </div>
        </div>
    }
}
