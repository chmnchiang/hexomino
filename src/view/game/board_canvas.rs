use std::{cell::RefCell, rc::Rc};

use anyhow::{Context as _, Result};
use gloo_render::AnimationFrame;
use log::{debug, error};
use piet::{
    kurbo::{Affine, Line, Point, Rect, Vec2},
    Color, RenderContext,
};
use piet_web::{Brush, WebRenderContext};
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent, MouseEvent};
use yew::{html, scheduler::Shared, Callback, Component, Context, NodeRef, Properties};

use crate::{
    game::{
        constants::{self, COLS, ROWS},
        hexo::{Hexo, MovedHexo, RHexo, Transform},
        pos::Pos,
        state::Player,
    },
    view::{
        util::SharedLink,
        window_events::{KeyDownListener, WindowResizeListener},
    },
};

use super::state::SharedGameViewState;

#[derive(Properties, PartialEq)]
pub struct BoardProps {
    pub state: SharedGameViewState,
    pub shared_link: SharedLink<BoardCanvas>,
    pub place_hexo_callback: Callback<MovedHexo>,
}

pub struct BoardCanvas {
    canvas: NodeRef,
    canvas_wrapper: NodeRef,
    renderer: Option<Shared<BoardRenderer>>,
    animation_handle: Option<AnimationFrame>,
    key_down_listener: Option<KeyDownListener>,
    window_resize_listener: Option<WindowResizeListener>,
}

pub enum BoardMsg {
    Select(Hexo),
    MouseMoved(Point),
    Clicked,
    KeyDown(String),
    WindowResize,
    MouseLeave,
}

impl BoardCanvas {
    fn relative_mouse_pos(&self, pos: Point) -> Option<Point> {
        let canvas = self.canvas.cast::<HtmlCanvasElement>()?;
        let rect = canvas.get_bounding_client_rect();
        let scale_x = canvas.width() as f64 / rect.width();
        let scale_y = canvas.height() as f64 / rect.height();
        let x = (pos.x - rect.left()) * scale_x;
        let y = (pos.y - rect.top()) * scale_y;
        Some((x, y).into())
    }
}

impl Component for BoardCanvas {
    type Properties = BoardProps;
    type Message = BoardMsg;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas: Default::default(),
            canvas_wrapper: Default::default(),
            renderer: None,
            animation_handle: None,
            key_down_listener: None,
            window_resize_listener: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use BoardMsg::*;

        guard::guard!(
            let Some(ref renderer) = self.renderer else {
                debug!("renderer is not ready");
                return false;
            }
        );

        {
            let mut renderer = renderer.borrow_mut();
            match msg {
                Select(hexo) => {
                    renderer.state.update_selected_hexo(hexo);
                }
                MouseMoved(point) => {
                    guard::guard!(
                        let Some(point) = self.relative_mouse_pos(point) else {
                            error!("can't get relative mouse position");
                            return false;
                        }
                    );
                    renderer.state.update_mouse_pos(point);
                }
                Clicked => {
                    let hexo = renderer.get_moved_hexo_on_click();
                    if let Some(moved_hexo) = hexo {
                        ctx.props().place_hexo_callback.emit(moved_hexo);
                        renderer.state.clear_selected_hexo();
                    }
                }
                KeyDown(event) => match event.as_str() {
                    "CapsLock" => renderer.state.flip(),
                    "Shift" => renderer.state.rotate(),
                    _ => (),
                },
                WindowResize => {
                    if let Some(canvas) = self.canvas.cast::<HtmlCanvasElement>() {
                        let (width, height) = resize_canvas_and_return_size(&canvas).unwrap();
                        renderer.state.set_width_height(width, height);
                    }
                }
                MouseLeave => {
                    renderer.state.clear_mouse_pos();
                }
            }
        }
        let renderer = renderer.clone();
        self.animation_handle = Some(gloo_render::request_animation_frame(move |_| {
            renderer.borrow_mut().render();
        }));
        false
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
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
        let renderer = BoardRenderer::create(context2d, Rc::clone(&ctx.props().state))
            .expect("can't create renderer");
        let renderer = Rc::new(RefCell::new(renderer));
        let (width, height) = resize_canvas_and_return_size(&canvas).unwrap();
        {
            let mut renderer = renderer.borrow_mut();
            renderer.state.set_width_height(width, height);
            renderer.render();
        }
        self.renderer = Some(renderer);
        let key_down_callback = ctx
            .link()
            .callback(|event: KeyboardEvent| BoardMsg::KeyDown(event.key()));
        self.key_down_listener = Some(KeyDownListener::register(key_down_callback).unwrap());
        let window_resize_callback = ctx.link().callback(|_| BoardMsg::WindowResize);
        self.window_resize_listener =
            Some(WindowResizeListener::register(window_resize_callback).unwrap());
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let onmousemove = ctx.link().callback(|event: MouseEvent| {
            Self::Message::MouseMoved((event.x() as f64, event.y() as f64).into())
        });
        let onclick = ctx.link().callback(|_| BoardMsg::Clicked);
        let onmouseleave = ctx.link().callback(|_| BoardMsg::MouseLeave);
        ctx.props().shared_link.install(ctx.link().clone());
        html! {
            <div ref={self.canvas_wrapper.clone()}>
                <canvas ref={self.canvas.clone()} style="width: 100%; height: 60vh" {onmousemove} {onclick} {onmouseleave}/>
                <p> {"<Shift> = Rotate, <CapsLock> = Flip"} </p>
            </div>
        }
    }
}

fn resize_canvas_and_return_size(canvas: &HtmlCanvasElement) -> Result<(u32, u32)> {
    let width = canvas.client_width() as u32;
    let height = canvas.client_height() as u32;
    canvas.set_width(width);
    canvas.set_height(height);
    Ok((width, height))
}

pub struct BoardRenderer {
    ctx: WebRenderContext<'static>,
    game_view_state: SharedGameViewState,
    state: RendererState,
}

pub struct RendererState {
    mouse_pos: Option<Point>,
    rhexo: Option<RHexo>,
    width: u32,
    height: u32,
}

impl RendererState {
    fn update_mouse_pos(&mut self, pos: Point) {
        self.mouse_pos = Some(pos);
    }
    fn clear_mouse_pos(&mut self) {
        self.mouse_pos = None;
    }
    fn update_selected_hexo(&mut self, hexo: Hexo) {
        self.rhexo = Some(hexo.apply(Transform::I));
    }
    fn clear_selected_hexo(&mut self) {
        self.rhexo = None;
    }
    fn flip(&mut self) {
        self.rhexo = self.rhexo.map(|rhexo| rhexo.flip());
    }
    fn rotate(&mut self) {
        self.rhexo = self.rhexo.map(|rhexo| rhexo.rotate());
    }
    fn set_width_height(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

const CANVAS_MARGIN: f64 = 20.0;
const BLOCK_LENGTH: f64 = 30.0;
const BLOCK_BORDER_COLOR: Color = Color::BLACK;
const BLOCK_BORDER_WIDTH: f64 = 2.0;
const BOARD_BORDER_WIDTH: f64 = 1.5;
const P1_BLOCK_COLOR: Color = Color::rgb8(32, 192, 0);
const P2_BLOCK_COLOR: Color = Color::rgb8(192, 32, 0);
const DEFAULT_BLOCK_COLOR: Color = Color::GRAY;

fn center_of_mass(hexo: &Hexo) -> Vec2 {
    let mut res = Vec2::ZERO;
    for point in hexo.tiles().map(Vec2::from) {
        res += point;
    }
    res / 6.0
}

fn player_to_color(player: Option<Player>) -> Color {
    match player {
        Some(Player::First) => P1_BLOCK_COLOR,
        Some(Player::Second) => P2_BLOCK_COLOR,
        None => DEFAULT_BLOCK_COLOR,
    }
}

#[derive(Clone, Copy)]
enum MousePos {
    Free(Point),
    Locked(Pos),
}

impl MousePos {
    fn from_point(point: Point) -> Self {
        use MousePos::*;
        let Point { x, y } = point;
        let i = (x / BLOCK_LENGTH).floor() as i32;
        let j = (y / BLOCK_LENGTH).floor() as i32;
        if i < 0 || i >= COLS as i32 || j < 0 || j >= ROWS as i32 {
            Free(point)
        } else {
            Locked(Pos::new(i, j))
        }
    }

    fn to_render_point(self) -> Point {
        use MousePos::*;
        match self {
            Free(point) => Point::new(point.x - BLOCK_LENGTH / 2.0, point.y - BLOCK_LENGTH / 2.0),
            Locked(Pos { x, y }) => Point::new(x as f64 * BLOCK_LENGTH, y as f64 * BLOCK_LENGTH),
        }
    }
}

impl BoardRenderer {
    pub fn create(
        ctx: CanvasRenderingContext2d,
        game_view_state: SharedGameViewState,
    ) -> Result<Self> {
        let window = web_sys::window().context("can't get window object")?;
        Ok(Self {
            ctx: WebRenderContext::new(ctx, window),
            game_view_state,
            state: RendererState {
                mouse_pos: None,
                rhexo: None,
                width: 0,
                height: 0,
            },
        })
    }

    fn clear(&mut self) {
        self.ctx.clear(None, Color::grey8(240));
    }

    pub fn render(&mut self) {
        self.clear();
        let transform = self.base_transform();
        self.with_affine(transform, |this| {
            this.render_board_tiles();
            let placed_hexos = this
                .game_view_state
                .borrow()
                .game_state
                .board()
                .placed_hexos()
                .to_vec();
            for hexo in placed_hexos {
                this.render_tiles(
                    hexo.moved_hexo().tiles(),
                    player_to_color(Some(hexo.player())),
                )
            }
            if let Some(mouse_pos) = this.state.mouse_pos {
                this.render_mouse(transform.inverse() * mouse_pos);
            }
        });
        self.ctx.finish().expect("render failed");
    }

    fn base_transform(&self) -> Affine {
        let width = self.state.width as f64;
        let height = self.state.height as f64;
        let block_len = ((width - CANVAS_MARGIN * 2.0) / COLS as f64)
            .min((height - CANVAS_MARGIN * 2.0) / ROWS as f64)
            .max(2.0);
        let scale = Affine::scale(block_len / BLOCK_LENGTH);

        let x_margin = (width - block_len * COLS as f64) / 2.0;
        let y_margin = (height - block_len * ROWS as f64) / 2.0;
        let translate = Affine::translate((x_margin, y_margin));

        translate * scale
    }

    pub fn render_tiles(&mut self, tiles: impl Iterator<Item = Pos>, color: Color) {
        let fill = self.ctx.solid_brush(color);
        for tile in tiles {
            let x = tile.x as f64 * BLOCK_LENGTH;
            let y = tile.y as f64 * BLOCK_LENGTH;
            self.render_block(Point::new(x, y), &fill);
        }
    }

    pub fn render_block(&mut self, point: Point, fill: &Brush) {
        let border_brush = self.ctx.solid_brush(BLOCK_BORDER_COLOR);
        self.ctx.fill(
            Rect::new(
                point.x,
                point.y,
                point.x + BLOCK_LENGTH,
                point.y + BLOCK_LENGTH,
            ),
            fill,
        );
        self.ctx.stroke(
            Rect::new(
                point.x,
                point.y,
                point.x + BLOCK_LENGTH,
                point.y + BLOCK_LENGTH,
            ),
            &border_brush,
            BLOCK_BORDER_WIDTH,
        );
    }

    pub fn with_affine(&mut self, affine: Affine, func: impl FnOnce(&mut Self)) {
        self.ctx.save().unwrap();
        self.ctx.transform(affine);
        func(self);
        self.ctx.restore().unwrap();
    }

    pub fn with_translate<PT: Into<Vec2>>(&mut self, translate: PT, func: impl FnOnce(&mut Self)) {
        self.with_affine(Affine::translate(translate), func)
    }

    pub fn render_board_tiles(&mut self) {
        // Render COLS x ROWS
        let border_brush = self.ctx.solid_brush(Color::grey8(30));
        let fill_brush = self.ctx.solid_brush(Color::WHITE);
        self.ctx.fill(
            Rect::new(
                0.0,
                0.0,
                BLOCK_LENGTH * COLS as f64,
                BLOCK_LENGTH * ROWS as f64,
            ),
            &fill_brush,
        );
        for i in 0..=COLS {
            self.ctx.stroke(
                Line::new(
                    Point::new(BLOCK_LENGTH * (i as f64), 0.0),
                    Point::new(BLOCK_LENGTH * (i as f64), BLOCK_LENGTH * (ROWS as f64)),
                ),
                &border_brush,
                BOARD_BORDER_WIDTH,
            )
        }
        for i in 0..=ROWS {
            self.ctx.stroke(
                Line::new(
                    Point::new(0.0, BLOCK_LENGTH * (i as f64)),
                    Point::new(BLOCK_LENGTH * (COLS as f64), BLOCK_LENGTH * (i as f64)),
                ),
                &border_brush,
                BOARD_BORDER_WIDTH,
            )
        }
    }

    fn render_mouse(&mut self, mouse_point: Point) {
        guard::guard!(let Some(rhexo) = self.state.rhexo else { return });
        let mouse_pos = MousePos::from_point(mouse_point);
        let real_point = mouse_pos.to_render_point();
        let current_player = self.game_view_state.borrow().game_state.current_player();
        self.with_translate((real_point.x, real_point.y), |this| {
            this.render_tiles(rhexo.tiles(), player_to_color(current_player));
        })
    }

    fn get_moved_hexo_on_click(&self) -> Option<MovedHexo> {
        let transformed_mouse_point = self.base_transform().inverse() * self.state.mouse_pos?;
        let rhexo = self.state.rhexo?;
        guard::guard!(let MousePos::Locked(pos) =
            MousePos::from_point(transformed_mouse_point) else { return None });
        let moved_hexo = rhexo.move_to(pos);
        if self
            .game_view_state
            .borrow()
            .game_state
            .board()
            .can_place(&moved_hexo)
        {
            Some(moved_hexo)
        } else {
            None
        }
    }
}
