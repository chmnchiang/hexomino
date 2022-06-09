use hexomino_core::{Hexo, MovedHexo};
use yew::{html, Callback, Component, Context, Html, Properties};

use super::{bottom_message::BottomMessage};
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
        let Some(_current_player) = core.current_player()
            else { return html!() };
        let select_onclick = ctx.link().callback(PlaceAction::Select);
        let shared_link = SharedLink::<BoardCanvas>::new();
        ctx.link()
            .send_message(PlaceAction::SetLink(shared_link.downgrade()));

        let place_hexo_callback = ctx.link().callback(PlaceAction::Placed);

        let me = state.me();
        let my_hexos = core.inventory().hexos_of(me).iter();
        let my_styled_hexos = my_hexos
            .map(|hexo| {
                (
                    hexo,
                    if Some(hexo) == self.state.selected_hexo {
                        Some("my-picked-hexo".to_string())
                    } else {
                        None
                    },
                )
            })
            .collect::<Vec<StyledHexo>>();
        let their_hexos = core.inventory().hexos_of(me.other()).iter();
        let their_styled_hexos = their_hexos
            .map(|hexo| (hexo, None))
            .collect::<Vec<StyledHexo>>();

        let my_turn = state.core().current_player() == Some(state.me());

        html! {
            <>
                <BoardCanvas state={ctx.props().state.clone()} {shared_link} {place_hexo_callback}/>
                <HexoTable styled_hexos={my_styled_hexos} on_hexo_click={select_onclick} owner_is_me={Some(true)}/>
                <div style="margin-top: 10px"></div>
                <HexoTable styled_hexos={their_styled_hexos} owner_is_me={Some(false)}/>
                <BottomMessage> {
                    if my_turn {
                        html! {
                            <p style="font-size: 1.5rem">
                                <b> {"Your turn: "} </b>
                                <span> { "Try to place a hexomino on the board." } </span>
                                <ol style="list-style-position: inside;">
                                    <li> {"Select a hexomino to place by clicking on its block."} </li>
                                    <li>
                                        <span> {"Move your mouse to the position to be placed. Press"} </span>
                                        <div style="display: inline-block; border: 2px solid #222222; border-radius: 5px;
                                            margin-left: 5px; margin-right: 5px; padding-left: 3px; padding-right: 3px;
                                            margin-bottom: 1px; color: #222222; letter-spacing: -1px;">{"Caps Lock"}</div>
                                        <span> {"to flip the hexomino horizontally. Press"} </span>
                                        <div style="display: inline-block; border: 2px solid #222222; border-radius: 5px;
                                            margin-left: 5px; margin-right: 5px; padding-left: 3px; padding-right: 8px;
                                            margin-top: 1px; color: #222222; letter-spacing: -1px;">{"⇧Shift"}</div>
                                        <span> {"to rotate the hexomino counter-clockwise."} </span>
                                    </li>
                                </ol>
                            </p>
                        }
                    } else {
                        html!{
                            <p style="font-size: 1.5rem">
                                <b> {"Opponent's turn: "} </b>
                                <span> { "Wait for your opponent to place a hexomino on the board" } </span>
                            </p>
                        }
                    }
                } </BottomMessage>
            </>
        }
    }
}
