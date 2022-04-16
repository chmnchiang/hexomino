use guard::guard;
use hexomino_core::{
    constants::{COLS, ROWS},
    Board, Hexo, PlacedHexo, Player, Pos, RHexo,
};
use piet::{
    kurbo::{Affine, Line, Point, Rect, Vec2},
    Color, RenderContext,
};
use piet_web::{Brush, WebRenderContext};

use crate::game::SharedGameState;

pub struct BoardRenderer<'a> {
    ctx: &'a mut WebRenderContext<'static>,
    config: RenderConfig,
}

pub struct RenderConfig {
    pub width: f64,
    pub height: f64,
    pub game_view_state: SharedGameState,
    pub mouse_point: Option<Point>,
    pub rhexo: Option<RHexo>,
}

const CANVAS_MARGIN: f64 = 20.0;
const BLOCK_LENGTH: f64 = 30.0;
const BLOCK_INNER_BORDER_WIDTH: f64 = 2.0;
const BLOCK_INNER_BORDER_COLOR: Color = Color::grey8(0x60);
const BLOCK_OUTER_BORDER_WIDTH: f64 = 3.0;
const BLOCK_OUTER_BORDER_COLOR: Color = Color::BLACK;
const BLOCK_OUTER_BORDER_LAST_COLOR: Color = Color::rgb8(32, 32, 255);
const BOARD_BORDER_WIDTH: f64 = 1.5;
const P1_BLOCK_COLOR: Color = Color::rgb8(32, 192, 0);
const P2_BLOCK_COLOR: Color = Color::rgb8(192, 32, 0);
const P1_BLOCK_LAST_COLOR: Color = Color::rgb8(48, 240, 32);
const P2_BLOCK_LAST_COLOR: Color = Color::rgb8(240, 48, 32);
const INVALID_BLOCK_COLOR: Color = Color::rgb8(240, 240, 64);
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

fn player_to_color_last(player: Option<Player>) -> Color {
    match player {
        Some(Player::First) => P1_BLOCK_LAST_COLOR,
        Some(Player::Second) => P2_BLOCK_LAST_COLOR,
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

impl<'a> BoardRenderer<'a> {
    pub fn new(ctx: &'a mut WebRenderContext<'static>, config: RenderConfig) -> Self {
        Self { ctx, config }
    }

    pub fn clear(&mut self) {
        self.ctx.clear(None, Color::grey8(240));
    }

    pub fn render(&mut self) {
        self.clear();
        let transform = Self::base_transform(self.config.width, self.config.height);
        self.with_affine(transform, |this| {
            this.render_board_tiles();
            this.render_hexos_on_board();
            this.render_mouse(transform);
        });
        self.ctx.finish().expect("render failed");
    }

    pub fn get_click_pos(width: f64, height: f64, mouse_point: Point) -> Option<Pos> {
        let transformed_mouse_point =
            BoardRenderer::base_transform(width, height).inverse() * mouse_point;
        match MousePos::from_point(transformed_mouse_point) {
            MousePos::Locked(pos) => Some(pos),
            MousePos::Free(..) => None,
        }
    }

    fn render_board_tiles(&mut self) {
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

    fn render_hexos_on_board(&mut self) {
        let placed_hexos = self
            .config
            .game_view_state
            .borrow()
            .core_game_state
            .board()
            .placed_hexos()
            .to_vec();
        for (i, hexo) in placed_hexos.iter().enumerate() {
            let is_last = i == placed_hexos.len() - 1;
            self.render_placed_hexos(hexo, is_last);
        }
    }

    fn render_mouse(&mut self, transform: Affine) {
        guard!(let Some(mouse_point) = self.config.mouse_point else { return });
        guard!(let Some(rhexo) = self.config.rhexo else { return });
        let mouse_pos = MousePos::from_point(transform.inverse() * mouse_point);
        let real_point = mouse_pos.to_render_point();
        match mouse_pos {
            MousePos::Locked(pos) => {
                self.render_placed_hexos_with_conflict(rhexo.move_to(pos).placed_by(Player::First));
            }
            MousePos::Free(_) => {
                self.with_translate((real_point.x, real_point.y), |this| {
                    this.render_placed_hexos(
                        &rhexo.move_to(Pos::ZERO).placed_by(Player::First),
                        false,
                    );
                });
            }
        }
    }

    fn base_transform(width: f64, height: f64) -> Affine {
        let block_len = ((width - CANVAS_MARGIN * 2.0) / COLS as f64)
            .min((height - CANVAS_MARGIN * 2.0) / ROWS as f64)
            .max(2.0);
        let scale = Affine::scale(block_len / BLOCK_LENGTH);

        let x_margin = (width - block_len * COLS as f64) / 2.0;
        let y_margin = (height - block_len * ROWS as f64) / 2.0;
        let translate = Affine::translate((x_margin, y_margin));

        translate * scale
    }

    fn render_placed_hexos(&mut self, placed_hexos: &PlacedHexo, is_last: bool) {
        let moved_hexo = placed_hexos.moved_hexo();
        for tile in moved_hexo.tiles() {
            let x = tile.x as f64 * BLOCK_LENGTH;
            let y = tile.y as f64 * BLOCK_LENGTH;
            let player = placed_hexos.player();
            let fill_color = if is_last {
                player_to_color_last(Some(player))
            } else {
                player_to_color(Some(player))
            };
            let fill = self
                .ctx
                .solid_brush(fill_color);
            self.render_block(Point::new(x, y), &fill);
        }
        let brush = if is_last {
            self.ctx.solid_brush(BLOCK_OUTER_BORDER_LAST_COLOR)
        } else {
            self.ctx.solid_brush(BLOCK_OUTER_BORDER_COLOR)
        };
        self.render_borders(moved_hexo.borders(), &brush);
    }

    fn render_placed_hexos_with_conflict(&mut self, placed_hexos: PlacedHexo) {
        let game_view_state = self.config.game_view_state.clone();
        let game_view_state = game_view_state.borrow();
        let board = game_view_state.core_game_state.board();
        let moved_hexo = placed_hexos.moved_hexo();
        for tile in moved_hexo.tiles() {
            let x = tile.x as f64 * BLOCK_LENGTH;
            let y = tile.y as f64 * BLOCK_LENGTH;
            let color = if !Board::in_bound(tile) || board.is_placed(tile) {
                INVALID_BLOCK_COLOR
            } else {
                player_to_color(Some(placed_hexos.player()))
            };
            let fill = self.ctx.solid_brush(color);
            self.render_block(Point::new(x, y), &fill);
        }
        let brush = &self.ctx.solid_brush(BLOCK_OUTER_BORDER_COLOR);
        self.render_borders(moved_hexo.borders(), brush);
    }

    fn render_locked_rhexo(&mut self, rhexo: &RHexo, color: Color) {
        let fill = self.ctx.solid_brush(color);
        for tile in rhexo.tiles() {
            let x = tile.x as f64 * BLOCK_LENGTH;
            let y = tile.y as f64 * BLOCK_LENGTH;
            self.render_block(Point::new(x, y), &fill);
        }
        let brush = &self.ctx.solid_brush(BLOCK_OUTER_BORDER_COLOR);
        self.render_borders(rhexo.borders(), brush);
    }

    fn render_block(&mut self, point: Point, fill: &Brush) {
        let border_brush = self.ctx.solid_brush(BLOCK_INNER_BORDER_COLOR);
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
            BLOCK_INNER_BORDER_WIDTH,
        );
    }

    fn render_borders(&mut self, borders: impl Iterator<Item = (Pos, Pos)>, border_brush: &Brush) {
        for (p1, p2) in borders {
            let x1 = p1.x as f64 * BLOCK_LENGTH;
            let y1 = p1.y as f64 * BLOCK_LENGTH;
            let x2 = p2.x as f64 * BLOCK_LENGTH;
            let y2 = p2.y as f64 * BLOCK_LENGTH;
            self.ctx.stroke(
                Line::new((x1, y1), (x2, y2)),
                border_brush,
                BLOCK_OUTER_BORDER_WIDTH,
            );
        }
    }

    fn with_affine(&mut self, affine: Affine, func: impl FnOnce(&mut Self)) {
        self.ctx.save().unwrap();
        self.ctx.transform(affine);
        func(self);
        self.ctx.restore().unwrap();
    }

    fn with_translate<PT: Into<Vec2>>(&mut self, translate: PT, func: impl FnOnce(&mut Self)) {
        self.with_affine(Affine::translate(translate), func)
    }
}
