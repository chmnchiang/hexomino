use gloo::{events::EventListener, utils::window};
use hexomino_core::Player;
use piet_web::WebRenderContext;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{
    classes, function_component, html, use_effect_with_deps, use_mut_ref, use_node_ref,
    use_ref, Properties,
};

use crate::{game::SharedGameState, view::util::resize_canvas_and_return_size};

use super::board_renderer::{BoardRenderer, RenderConfig};

#[derive(PartialEq, Properties)]
pub struct EndViewProps {
    pub state: SharedGameState,
}

struct RenderState {
    width: f64,
    height: f64,
}

#[function_component(EndView)]
pub fn end_view(props: &EndViewProps) -> Html {
    let state = props.state.borrow();
    let winner = state.core_game_state.winner().unwrap();
    let canvas_ref = use_node_ref();
    let web_render_context = use_mut_ref(|| None);
    let render_func = {
        let canvas_ref = canvas_ref.clone();
        let web_render_context = web_render_context.clone();
        let game_view_state = props.state.clone();
        use_ref(move || {
            move || {
                let canvas: HtmlCanvasElement = canvas_ref.cast().unwrap();
                let mut web_render_context = web_render_context.borrow_mut();
                let (width, height) = resize_canvas_and_return_size(&canvas);
                BoardRenderer::new(
                    web_render_context.as_mut().unwrap(),
                    RenderConfig {
                        width,
                        height,
                        game_view_state: game_view_state.clone(),
                        mouse_point: None,
                        rhexo: None,
                    },
                ).render();
            }
        })
    };
    let _window_resize_listener = use_ref({
        let render_func = render_func.clone();
        || {
            EventListener::new(&window(), "resize", move |_| {
                render_func();
            })
        }
    });
    {
        let canvas_ref = canvas_ref.clone();
        let web_render_context = web_render_context.clone();
        let render_func = render_func.clone();
        use_effect_with_deps(
            move |_| {
                let canvas: HtmlCanvasElement = canvas_ref.cast().unwrap();
                let context2d: CanvasRenderingContext2d = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into()
                    .unwrap();
                *web_render_context.borrow_mut() = Some(WebRenderContext::new(context2d, window()));
                render_func();

                || ()
            },
            (),
        );
    }
    let style = match winner {
        Player::First => "my-foreground",
        Player::Second => "their-foreground",
    };
    html! {
        <>
            <div class="columns is-mobile is-centered">
                <div class="column is-narrow">
                    <h1 class={classes!("title", style)}>{ format!("{} Wins", state.name_of(winner)) }</h1>
                </div>
            </div>
            <div class="columns is-centered">
                <div class="column">
                    <canvas ref={canvas_ref} style="width: 100%; height: 60vh"/>
                </div>
            </div>
        </>
    }
}
