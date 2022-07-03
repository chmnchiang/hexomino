use api::GameEndReason;
use gloo::{events::EventListener, utils::window};


use piet_web::WebRenderContext;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{
    function_component, html, use_effect_with_deps, use_mut_ref, use_node_ref, use_ref, Properties, use_state, Callback,
};

use crate::{game::SharedGameState, view::util::resize_canvas_and_return_size};

use super::{board_renderer::{BoardRenderer, RenderConfig}, hexo_table::{HexoTable, StyledHexo}};

#[derive(PartialEq, Properties)]
pub struct EndViewProps {
    pub state: SharedGameState,
    #[prop_or(["Player 1".to_string(), "Player 2".to_string()])]
    pub names: [String; 2],
    #[prop_or(0)]
    pub nonce: usize,
}

struct RenderState {
    width: f64,
    height: f64,
}

#[function_component(EndView)]
pub fn end_view(props: &EndViewProps) -> Html {
    let state = props.state.borrow();
    let end_state = state.end_state().clone().expect("game not ended");

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
                )
                .render();
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
    let me = state.me();
    let winner = end_state.winner;
    let i_won = me == winner;
    let show_my_hexo = use_state(|| true);
    let my_hexo_tab_onclick = {
        let show_my_hexo = show_my_hexo.clone();
        Callback::from(move |_| show_my_hexo.set(true))
    };
    let their_hexo_tab_onclick = {
        let show_my_hexo = show_my_hexo.clone();
        Callback::from(move |_| show_my_hexo.set(false))
    };
    let my_styled_hexos = state.core().inventory().hexos_of(me)
        .iter()
        .map(|hexo| (hexo, None))
        .collect::<Vec<StyledHexo>>();
    let their_styled_hexos = state.core().inventory().hexos_of(me.other())
        .iter()
        .map(|hexo| (hexo, None))
        .collect::<Vec<StyledHexo>>();
    let reason_str = format!("Reason: {}", match end_state.reason {
        GameEndReason::NoValidMove => "No valid move",
        GameEndReason::TimeLimitExceed => "Time limit exceeded",
    });

    html! {
        <>
            <div class="columns is-mobile is-centered" style="margin-bottom: 0px">
                <div class="column is-narrow">
                    if i_won {
                        <h1 class="title my-foreground">
                            <span class="icon" style="margin-right: 10px;">
                                <i class="fas fa-trophy"></i>
                            </span>
                            <span> { "You won this game" } </span>
                        </h1>

                    } else {
                        <h1 class="title their-foreground">{ "Your opponent won this game" }</h1>
                    }
                    <h2 class="subtitle" style="text-align: center"> {reason_str} </h2>
                </div>
            </div>
            <div class="columns is-centered">
                <div class="column">
                    <canvas ref={canvas_ref} style="width: 100%; height: 60vh"/>
                </div>
            </div>
            <div class="tabs is-centered is-toggle" style="margin-bottom: 0.5rem">
                <ul>
                    <li class={show_my_hexo.then_some("is-active")} onclick={my_hexo_tab_onclick}>
                        <a>{"Your Hexominos"}</a>
                    </li>
                    <li class={(!*show_my_hexo).then_some("is-active")} onclick={their_hexo_tab_onclick}>
                        <a>{"Opponent's Hexominos"}</a>
                    </li>
                </ul>
            </div>
            if *show_my_hexo {
                <HexoTable styled_hexos={my_styled_hexos}/>
            } else {
                <HexoTable styled_hexos={their_styled_hexos}/>
            }
        </>
    }
}
