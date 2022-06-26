use std::iter::repeat;

use hexomino_core::Hexo;
use itertools::Itertools;
use yew::{function_component, html, Callback, Html, Properties};

use super::hexo_block::HexoBlock;

pub type StyledHexo = (Hexo, Option<String>);

#[derive(PartialEq, Properties)]
pub struct HexoTableProps {
    pub styled_hexos: Vec<StyledHexo>,
    #[prop_or_default]
    pub on_hexo_click: Callback<Hexo>,
    #[prop_or(None)]
    pub owner_is_me: Option<bool>,
}

#[function_component(HexoTable)]
pub fn hexo_table(props: &HexoTableProps) -> Html {
    const CHUNK_SIZE: usize = 9;

    fn hexo_to_html(hexo: Hexo, style: Option<String>, onclick: Callback<Hexo>) -> Html {
        html! {
            <div class="square-block hexo-block">
                <HexoBlock {hexo} {style} onclick={onclick.clone()}/>
            </div>
        }
    }

    html! {
        <>
            {
                if let Some(is_me) = props.owner_is_me {
                    if is_me {
                        html! {
                            <div class="my-hexo-table-tag">
                                <span> { "Your hexominos" } </span>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="their-hexo-table-tag">
                                <span> { "Opponent's hexominos" } </span>
                            </div>
                        }
                    }
                } else {
                    html!()
                }
            }
            <div class="hexo-table-wrapper">
                <div style="width: 100%;">
                    <ul class="hexo-grid" style="width: 100%;"> {
                        props.styled_hexos
                            .iter()
                            .map(|(hexo, style)| hexo_to_html(*hexo, style.clone(), props.on_hexo_click.clone()))
                            .collect::<Html>()
                    } </ul>
                </div>
            </div>
        </>
    }
}
