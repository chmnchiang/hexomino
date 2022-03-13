use yew::{function_component, html, Html, Properties};
use crate::game::{state::{Inventory, Player}, hexo::Hexo};

use super::hexo_svg::HexoSvg;

#[derive(Properties, PartialEq)]
pub struct PickInventoryProps {
    pub inventory: Inventory,
    pub me: Player,
    //pub 
}


#[function_component(PickInventory)]
pub fn pick_phase(props: &PickInventoryProps) -> Html {
    html! {
        <div class="columns is-multiline">
        {
            Hexo::all_hexos().map(|hexo| html!{
                <div class="column" style="width: 10%; flex: none;">
                    <HexoSvg hexo={hexo}/>
                </div>
            }).collect::<Html>()
        }
        </div>
    }
}

