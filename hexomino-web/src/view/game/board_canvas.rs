use std::{cell::RefCell, rc::Rc};

use gloo::{
    events::EventListener,
    render::{request_animation_frame, AnimationFrame},
    utils::{document, window},
};
use hexomino_core::{Board, Hexo, MovedHexo, RHexo, Transform};
use log::{debug, error};
use piet::kurbo::Point;
use piet_web::WebRenderContext;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent, MouseEvent};
use yew::{html, scheduler::Shared, Callback, Component, Context, NodeRef, Properties};

use crate::{
    game::SharedGameState,
    view::{
        game::board_renderer::{BoardRenderer, RenderConfig},
        shared_link::SharedLink,
        util::resize_canvas_and_return_size,
    },
};

#[derive(Properties, PartialEq)]
pub struct BoardProps {
    pub state: SharedGameState,
    pub shared_link: SharedLink<BoardCanvas>,
    pub place_hexo_callback: Callback<MovedHexo>,
}

pub struct BoardCanvas {
    canvas: NodeRef,
    web_render_context: Option<Shared<WebRenderContext<'static>>>,
    render_state: RenderState,
    animation_handle: Option<AnimationFrame>,
    key_down_listener: Option<EventListener>,
    window_resize_listener: Option<EventListener>,
}

#[derive(Clone)]
pub struct RenderState {
    mouse_point: Option<Point>,
    rhexo: Option<RHexo>,
    width: f64,
    height: f64,
}

pub enum BoardMsg {
    Select(Hexo),
    MouseMoved(Point),
    Clicked,
    KeyDown(String),
    WindowResize,
    MouseLeave,
    ShouldRender,
}

impl BoardCanvas {
    fn relative_mouse_point(&self, point: Point) -> Option<Point> {
        let canvas = self.canvas.cast::<HtmlCanvasElement>()?;
        let rect = canvas.get_bounding_client_rect();
        let scale_x = canvas.width() as f64 / rect.width();
        let scale_y = canvas.height() as f64 / rect.height();
        let x = (point.x - rect.left()) * scale_x;
        let y = (point.y - rect.top()) * scale_y;
        Some((x, y).into())
    }

    fn get_moved_hexo_on_click(&self, board: &Board) -> Option<MovedHexo> {
        let rhexo = self.render_state.rhexo?;
        let pos = BoardRenderer::get_click_pos(
            self.render_state.width as f64,
            self.render_state.height as f64,
            self.render_state.mouse_point?,
        )?;
        let moved_hexo = rhexo.move_to(pos);
        if board.can_place(&moved_hexo) {
            Some(moved_hexo)
        } else {
            None
        }
    }
}

impl Component for BoardCanvas {
    type Properties = BoardProps;
    type Message = BoardMsg;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas: Default::default(),
            web_render_context: None,
            render_state: RenderState {
                mouse_point: None,
                rhexo: None,
                width: 0.0,
                height: 0.0,
            },
            animation_handle: None,
            key_down_listener: None,
            window_resize_listener: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use BoardMsg::*;

        let Some(ref web_render_context) = self.web_render_context else {
            debug!("web_render_context is not ready");
            return false;
        };

        {
            match msg {
                Select(hexo) => {
                    self.render_state.rhexo = Some(hexo.apply(Transform::I));
                }
                MouseMoved(point) => {
                    let Some(point) = self.relative_mouse_point(point) else {
                        error!("can't get relative mouse position");
                        return false;
                    };
                    self.render_state.mouse_point = Some(point);
                }
                Clicked => {
                    let state = ctx.props().state.borrow();
                    let board = state.core_game_state.board();
                    if let Some(moved_hexo) = self.get_moved_hexo_on_click(board) {
                        ctx.props().place_hexo_callback.emit(moved_hexo);
                        self.render_state.rhexo = None;
                    }
                }
                KeyDown(event) => {
                    if let Some(rhexo) = self.render_state.rhexo.as_mut() {
                        match event.as_str() {
                            "CapsLock" => *rhexo = rhexo.flip(),
                            "Shift" => *rhexo = rhexo.rotate(),
                            _ => (),
                        }
                    }
                }
                WindowResize => {
                    if let Some(canvas) = self.canvas.cast::<HtmlCanvasElement>() {
                        (self.render_state.width, self.render_state.height) =
                            resize_canvas_and_return_size(&canvas);
                    }
                }
                MouseLeave => {
                    self.render_state.mouse_point = None;
                }
                ShouldRender => (),
            }
        }

        let web_render_context = web_render_context.clone();
        let game_view_state = ctx.props().state.clone();
        let RenderState {
            mouse_point,
            rhexo,
            width,
            height,
        } = self.render_state.clone();
        self.animation_handle = Some(request_animation_frame(move |_| {
            let config = RenderConfig {
                width: width as f64,
                height: height as f64,
                game_view_state,
                mouse_point,
                rhexo,
            };
            let mut web_render_context = web_render_context.borrow_mut();
            let mut renderer = BoardRenderer::new(&mut *web_render_context, config);
            renderer.render();
        }));
        false
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            ctx.link().send_message(BoardMsg::ShouldRender);
            return;
        }

        let canvas = self
            .canvas
            .cast::<HtmlCanvasElement>()
            .expect("cannot convert to canvas");
        let context2d: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        let web_render_context = WebRenderContext::new(context2d, window());
        self.web_render_context = Some(Rc::new(RefCell::new(web_render_context)));

        let link = ctx.link().clone();
        self.key_down_listener = Some(EventListener::new(&document(), "keydown", move |event| {
            let event: &KeyboardEvent = event.dyn_ref().unwrap_throw();
            link.send_message(BoardMsg::KeyDown(event.key()))
        }));

        let link = ctx.link().clone();
        self.window_resize_listener = Some(EventListener::new(&window(), "resize", move |_| {
            link.send_message(BoardMsg::WindowResize)
        }));

        ctx.link().send_message(BoardMsg::WindowResize);
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let onmousemove = ctx.link().callback(|event: MouseEvent| {
            Self::Message::MouseMoved((event.x() as f64, event.y() as f64).into())
        });
        let onclick = ctx.link().callback(|_| BoardMsg::Clicked);
        let onmouseleave = ctx.link().callback(|_| BoardMsg::MouseLeave);
        ctx.props().shared_link.install(ctx.link().clone());
        html! {
            <div>
                <canvas ref={self.canvas.clone()} style="width: 100%; height: 60vh" {onmousemove} {onclick} {onmouseleave}/>
                <p> {"<Shift> = Rotate, <CapsLock> = Flip"} </p>
            </div>
        }
    }
}
