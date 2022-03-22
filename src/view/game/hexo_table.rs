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

        html! {
            <tr> {
                chunk.map(|hexo| html!{
                    <td class="square-td">
                        <div class="content" style="margin: 3px"> {
                            match hexo {
                                Some((hexo, style)) => html!{
                                    <HexoSvg hexo={*hexo} style={style.clone()} onclick={onclick.clone()}/>
                                },
                                None => html!{},
                            }
                        } </div>
                    </td>
                }).collect::<Html>()
            } </tr>
        }
    }

    html! {
        <div>
            <table style="width: 90%;"> {
                props.styled_hexos
                    .iter()
                    .chunks(CHUNK_SIZE)
                    .into_iter()
                    .map(|chunk| hexo_chunk_html(chunk, props.on_hexo_click.clone()))
                    .collect::<Html>()
            } </table>
        </div>
    }
}
