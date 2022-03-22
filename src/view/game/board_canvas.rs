use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use anyhow::{Context as _, Result};
use log::{debug, error};
use piet::{
    kurbo::{Affine, Line, Point, Rect, Vec2},
    Color, RenderContext,
};
use piet_web::{Brush, WebRenderContext};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, Window};
use yew::{html, scheduler::Shared, Callback, Component, Context, NodeRef, Properties};

use crate::game::{
    board::Board,
    constants::{self, COLS, ROWS},
    hexo::{Hexo, MovedHexo, RHexo, Transform},
    pos::Pos,
    state::Player,
};

use super::state::SharedGameViewState;

#[derive(Properties, PartialEq)]
pub struct BoardProps {
    pub selected_hexo: Option<Hexo>,
    pub state: SharedGameViewState,
    pub place_hexo_callback: Callback<MovedHexo>,
}

pub struct BoardCanvas {
    canvas: NodeRef,
    renderer: Option<Shared<BoardRenderer>>,
}

pub enum BoardMsg {
    Select(Hexo),
    MouseMoved(Point),
    Clicked,
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

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            canvas: Default::default(),
            renderer: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use BoardMsg::*;
        match msg {
            Select(hexo) => (),
            MouseMoved(point) => {
                guard::guard!(
                    let Some(point) = self.relative_mouse_pos(point) else {
                        error!("can't get relative mouse position");
                        return false;
                    }
                );
                guard::guard!(
                    let Some(ref renderer) = self.renderer else {
                        debug!("renderer is not ready");
                        return false;
                    }
                );
                renderer.borrow_mut().state.update_mouse_pos(point);
            }
            Clicked => {
                guard::guard!(
                    let Some(ref renderer) = self.renderer else {
                        debug!("renderer is not ready");
                        return false;
                    }
                );
                if let Some(moved_hexo) = renderer.borrow().get_moved_hexo_on_click() {
                    ctx.props().place_hexo_callback.emit(moved_hexo);
                }
            }
        }
        false
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
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
            start_render_loop(Rc::downgrade(&renderer));
            self.renderer = Some(renderer);
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let width = (constants::COLS * 30 + 60).to_string();
        let height = (constants::ROWS * 30 + 60).to_string();
        let onmousemove = ctx.link().callback(|event: MouseEvent| {
            Self::Message::MouseMoved((event.x() as f64, event.y() as f64).into())
        });
        if let Some(ref renderer) = &mut self.renderer.as_ref() {
            let renderer_state = &mut renderer.borrow_mut().state;
            debug!("check {:?}", ctx.props().selected_hexo);
            if let Some(hexo) = ctx.props().selected_hexo {
                renderer_state.update_selected_hexo(hexo);
            } else {
                renderer_state.clear_selected_hexo();
            }
        }
        let onclick = ctx.link().callback(|_| BoardMsg::Clicked);

        html! {
            <canvas ref={self.canvas.clone()} width={width} height={height} {onmousemove} {onclick}/>
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
        unimplemented!();
    }
    fn rotate(&mut self) {
        unimplemented!();
    }
}

const BLOCK_LENGTH: f64 = 30.0;
const BLOCK_BORDER_COLOR: Color = Color::BLACK;
const BLOCK_BORDER_WIDTH: f64 = 2.0;
const P1_BLOCK_COLOR: Color = Color::rgb8(192, 32, 0);
const P2_BLOCK_COLOR: Color = Color::rgb8(32, 192, 0);
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

fn start_render_loop(renderer_weak: Weak<RefCell<BoardRenderer>>) {
    let handle = gloo_render::request_animation_frame(move |_| {
        let renderer = renderer_weak.upgrade();
        if let Some(renderer) = renderer {
            renderer.borrow_mut().render((0.0, 0.0).into());
            start_render_loop(renderer_weak);
            //gloo_render::request_animation_frame(move |_| start_render_loop(renderer_weak));
        }
    });
    std::mem::forget(handle);
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

    fn to_render_point(&self) -> Point {
        use MousePos::*;
        match self {
            Free(point) => Point::new(point.x - BLOCK_LENGTH / 2.0, point.y - BLOCK_LENGTH / 2.0),
            Locked(Pos { x, y }) => Point::new(*x as f64 * BLOCK_LENGTH, *y as f64 * BLOCK_LENGTH),
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

    //pub fn start_rendering(&self) {
    //gloo_render::request_animation_frame(|_| {
    //self.render((0.0, 0.0).into());
    //gloo_render::request_animation_frame(|_| self.render
    //});
    //}

    pub fn render(&mut self, shift: Vec2) {
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
            if let Some(rhexo) = this.state.rhexo {
                this.render_tiles(rhexo.tiles(), player_to_color(current_player));
            }
        })
    }

    fn get_moved_hexo_on_click(&self) -> Option<MovedHexo> {
        guard::guard!(let Some(rhexo) = self.state.rhexo else { return None });
        let mouse_pos = MousePos::from_point(self.state.mouse_pos);
        guard::guard!(let MousePos::Locked(pos) = MousePos::from_point(self.state.mouse_pos) else { return None });
        let moved_hexo = rhexo.move_to(pos);
        let state = self.game_view_state.borrow();
        let game_state = &state.game_state;
        if game_state.board().can_place(&moved_hexo) {
            Some(moved_hexo)
        } else {
            None
        }
    }
}
