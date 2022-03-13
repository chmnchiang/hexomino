use std::{cell::{RefCell, Cell}, rc::Rc};

use anyhow::Context as _;
use log::info;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{html, Component, Context, Html, NodeRef, Properties};

use self::{hexo_svg::HexoSvg, pick_inventory::PickInventory, turn_indicator::TurnIndicator};
use crate::{
    game::{
        hexo::{Hexo, MovedHexo},
        state::{Action, State, Player},
    },
    render::Renderer,
};

mod hexo_svg;
mod pick_inventory;
mod turn_indicator;
mod player;

#[derive(PartialEq, Properties)]
pub struct GameProps;

type SharedGameState = Rc<RefCell<State>>;

pub struct GameComponent {
    canvas: NodeRef,
    state: SharedGameState,
}

impl GameComponent {
    fn init_canvas(&self, ctx: &Context<Self>) -> anyhow::Result<()> {
        let window = web_sys::window().unwrap();
        let canvas = self
            .canvas
            .cast::<HtmlCanvasElement>()
            .context("cannot convert to canvas")?;
        let context2d: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        let mut renderer = Renderer::new(context2d, window);
        let shared_state = Rc::clone(&self.state);
        let link = ctx.link().clone();

        let future = async move {
            use crate::game::{hexo::Hexo, state::Action};
            use log::info;
            use std::time::Duration;
            for hexo in Hexo::all_hexos() {
                gloo_timers::future::sleep(Duration::from_millis(100)).await;
                link.send_message(Action::Pick { hexo });
            }
            fn next_placement(state: &State) -> Option<MovedHexo> {
                match state {
                    State::Place(ref place_state) => {
                        let placement = place_state
                            .inventory()
                            .hexos_of(place_state.current_player())
                            .iter()
                            .map(|hexo| place_state.board().try_find_placement(hexo))
                            .filter_map(|x| x)
                            .nth(0)
                            .expect("nowhere to place, but the game did not end");
                        Some(placement)
                    }
                    _ => None,
                }
            }
            while let Some(placement) = {
                let state = shared_state.borrow();
                next_placement(&*state)
            } {
                info!("place = {placement:?}");
                link.send_message(Action::Place { hexo: placement });
            }
        };
        //wasm_bindgen_futures::spawn_local(future);
        Ok(())
    }
}

impl Component for GameComponent {
    type Message = Action;
    type Properties = GameProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas: NodeRef::default(),
            state: Rc::new(RefCell::new(State::new())),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.state.borrow_mut().play(msg);
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.init_canvas(ctx);
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = self.state.borrow();
        info!("current player = {:?}", state.current_player());
        html! {
            <div>
            {
                match *state {
                    State::Pick(ref state) => html!{ <PickInventory inventory={state.inventory().clone()}/> },
                    _ => html!{},
                }
            }
                //<TurnIndicator current_player={state.current_player()}/>
            </div>
        }
        //<canvas ref={self.canvas.clone()} height="600" width="800"></canvas>
    }
}
