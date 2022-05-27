use hexomino_core::{Hexo, MovedHexo, Player};
use yew::{html, Callback, Component, Context, Html, Properties};

use super::turn_indicator::TurnIndicator;
use crate::{
    game::SharedGameState,
    view::{
        game::{
            board_canvas::{BoardCanvas, BoardMsg},
            hexo_table::{HexoTable, StyledHexo},
        },
        shared_link::{SharedLink, WeakLink},
    },
};

#[derive(Properties, PartialEq)]
pub struct PlaceViewProps {
    pub state: SharedGameState,
    pub send_place: Callback<MovedHexo>,
    pub is_locked: bool,
}

#[derive(Debug, Default)]
struct PlaceViewState {
    pub selected_hexo: Option<Hexo>,
}

pub enum PlaceAction {
    Select(Hexo),
    Placed(MovedHexo),
    SetLink(WeakLink<BoardCanvas>),
}

#[derive(Default)]
pub struct PlaceView {
    state: PlaceViewState,
    board_weak_link: WeakLink<BoardCanvas>,
}

impl Component for PlaceView {
    type Message = PlaceAction;
    type Properties = PlaceViewProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use PlaceAction::*;
        match msg {
            Select(hexo) => {
                if ctx.props().is_locked {
                    return false
                }
                self.state.selected_hexo = Some(hexo);
                self.board_weak_link
                    .upgrade()
                    .unwrap()
                    .get()
                    .send_message(BoardMsg::Select(hexo));
            }
            Placed(moved_hexo) => {
                ctx.props().send_place.emit(moved_hexo);
                self.state.selected_hexo = None;
            }
            SetLink(link) => {
                self.board_weak_link = link;
                return false;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = ctx.props().state.borrow();
        let core = state.core();
        let Some(current_player) = core.current_player()
            else { return html!() };
        let select_onclick = ctx.link().callback(PlaceAction::Select);
        let shared_link = SharedLink::<BoardCanvas>::new();
        ctx.link()
            .send_message(PlaceAction::SetLink(shared_link.downgrade()));

        let place_hexo_callback = ctx.link().callback(PlaceAction::Placed);

        let _me = state.me();
        let player_hexos = core.inventory().hexos_of(current_player).iter();
        let styled_hexos = if let Some(selected_hexo) = self.state.selected_hexo {
            player_hexos
                .map(|hexo| {
                    (
                        hexo,
                        match (hexo == selected_hexo, current_player) {
                            (false, _) => None,
                            (true, Player::First) => Some("my-picked-hexo".to_string()),
                            (true, Player::Second) => Some("their-picked-hexo".to_string()),
                        },
                    )
                })
                .collect::<Vec<StyledHexo>>()
        } else {
            player_hexos
                .map(|hexo| (hexo, None))
                .collect::<Vec<StyledHexo>>()
        };

        html! {
            <>
                <TurnIndicator current_player={core.current_player()}/>
                <BoardCanvas state={ctx.props().state.clone()} {shared_link} {place_hexo_callback}/>
                <HexoTable {styled_hexos} on_hexo_click={select_onclick}/>
            </>
        }
    }
}
