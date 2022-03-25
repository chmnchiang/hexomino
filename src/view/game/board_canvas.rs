use std::{cell::RefCell, rc::Rc};

use anyhow::{bail, Context as _, Result};
use gloo_render::AnimationFrame;
use log::{debug, error};
use piet::{
    kurbo::{Affine, Line, Point, Rect, Vec2},
    Color, RenderContext,
};
use piet_web::{Brush, WebRenderContext};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    window, CanvasRenderingContext2d, Event, HtmlCanvasElement, KeyboardEvent, MouseEvent,
};
use yew::{html, scheduler::Shared, Callback, Component, Context, NodeRef, Properties};

use crate::{
    game::{
        constants::{self, COLS, ROWS},
        hexo::{Hexo, MovedHexo, RHexo, Transform},
        pos::Pos,
        state::Player,
    },
    view::util::SharedLink,
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
    renderer: Option<Shared<BoardRenderer>>,
    animation_handle: Option<AnimationFrame>,
    key_down_listener: Option<KeyDownListener>,
}

pub enum BoardMsg {
    Select(Hexo),
    MouseMoved(Point),
    Clicked,
    KeyDown(String),
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
            renderer: None,
            animation_handle: None,
            key_down_listener: None,
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
            }
        }
        let renderer = renderer.clone();
        self.animation_handle = Some(gloo_render::request_animation_frame(move |_| {
            renderer.borrow_mut().render((0.0, 0.0).into());
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
        renderer.borrow_mut().render((0.0, 0.0).into());
        self.renderer = Some(renderer);
        let key_down_callback = ctx
            .link()
            .callback(|event: KeyboardEvent| BoardMsg::KeyDown(event.key()));
        self.key_down_listener = Some(KeyDownListener::register(key_down_callback).unwrap());
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let width = (constants::COLS * 30 + 60).to_string();
        let height = (constants::ROWS * 30 + 60).to_string();
        let onmousemove = ctx.link().callback(|event: MouseEvent| {
            Self::Message::MouseMoved((event.x() as f64, event.y() as f64).into())
        });
        let onclick = ctx.link().callback(|_| BoardMsg::Clicked);
        ctx.props().shared_link.install(ctx.link().clone());
        html! {
            <>
                <canvas ref={self.canvas.clone()} width={width} height={height} {onmousemove} {onclick}/>
                <p> {"<Shift> = Rotate, <CapsLock> = Flip"} </p>
            </>
        }
    }
}

pub struct BoardRenderer {
    ctx: WebRenderContext<'static>,
    game_view_state: SharedGameViewState,
    state: RendererState,
}

pub struct RendererState {
    mouse_pos: Point,
    rhexo: Option<RHexo>,
}

impl RendererState {
    fn update_mouse_pos(&mut self, pos: Point) {
        self.mouse_pos = pos;
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
}

const BLOCK_LENGTH: f64 = 30.0;
const BLOCK_BORDER_COLOR: Color = Color::BLACK;
const BLOCK_BORDER_WIDTH: f64 = 2.0;
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
                mouse_pos: Point::ZERO,
                rhexo: None,
            },
        })
    }

    fn clear(&mut self) {
        self.ctx.clear(None, Color::WHITE);
    }

    pub fn render(&mut self, shift: Vec2) {
        debug!("call render");
        self.clear();
        self.with_affine(Affine::translate(shift), |this| {
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
            this.render_mouse();
        });
        self.ctx.finish().expect("render failed");
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
        for i in 0..=COLS {
            self.ctx.stroke(
                Line::new(
                    Point::new(BLOCK_LENGTH * (i as f64), 0.0),
                    Point::new(BLOCK_LENGTH * (i as f64), BLOCK_LENGTH * (ROWS as f64)),
                ),
                &Brush::Solid(123),
                1.0,
            )
        }
        for i in 0..=ROWS {
            self.ctx.stroke(
                Line::new(
                    Point::new(0.0, BLOCK_LENGTH * (i as f64)),
                    Point::new(BLOCK_LENGTH * (COLS as f64), BLOCK_LENGTH * (i as f64)),
                ),
                &Brush::Solid(123),
                1.0,
            )
        }
    }

    fn render_mouse(&mut self) {
        guard::guard!(let Some(rhexo) = self.state.rhexo else { return });
        let mouse_pos = MousePos::from_point(self.state.mouse_pos);
        let real_point = mouse_pos.to_render_point();
        let current_player = self.game_view_state.borrow().game_state.current_player();
        self.with_translate((real_point.x - 0.0, real_point.y - 0.0), |this| {
            this.render_tiles(rhexo.tiles(), player_to_color(current_player));
        })
    }

    fn get_moved_hexo_on_click(&self) -> Option<MovedHexo> {
        guard::guard!(let Some(rhexo) = self.state.rhexo else { return None });
        guard::guard!(let MousePos::Locked(pos) =
            MousePos::from_point(self.state.mouse_pos) else { return None });
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

struct KeyDownListener {
    closure: Closure<dyn Fn(KeyboardEvent)>,
}

impl KeyDownListener {
    fn register(callback: Callback<KeyboardEvent>) -> Result<Self> {
        debug!("register");
        let closure = Closure::wrap(Box::new(move |e| {
            debug!("e = {:?}", e);
            callback.emit(e);
        }) as Box<dyn Fn(KeyboardEvent)>);
        if let Err(_) = window()
            .unwrap()
            .document()
            .unwrap()
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        {
            bail!("Can't add onkeydown event listener");
        }
        Ok(Self { closure })
    }
}

impl Drop for KeyDownListener {
    fn drop(&mut self) {
        let _ = window()
            .unwrap()
            .document()
            .unwrap()
            .remove_event_listener_with_callback(
                "onkeydown",
                self.closure.as_ref().unchecked_ref(),
            );
    }
}
