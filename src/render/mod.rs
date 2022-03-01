use log::info;
use piet::{
    kurbo::{Affine, Line, Point, Rect, Vec2},
    Color, RenderContext,
};
use piet_web::{Brush, WebRenderContext};
use web_sys::{CanvasRenderingContext2d, Window};

use crate::game::{
    constants::{COLS, HEXOS, N_HEXOS, ROWS},
    hexo::{Hexo, MovedHexo, RHexo, Transform},
    point::Point as Coordinate,
    state::{GamePhase, Player, State},
};

pub struct Renderer {
    ctx: WebRenderContext<'static>,
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

impl Renderer {
    pub fn new(ctx: CanvasRenderingContext2d, window: Window) -> Self {
        Self {
            ctx: WebRenderContext::new(ctx, window),
        }
    }

    pub fn render<PT: Into<Vec2>>(&mut self, state: &State, shift: PT) {
        self.clear();
        self.with_affine(Affine::translate(shift), |this| {
            this.render_game_state(state);
        });
        self.ctx.finish().expect("render failed");
    }

    fn clear(&mut self) {
        self.ctx.clear(None, Color::WHITE);
    }

    pub fn render_game_state(&mut self, state: &State) {
        match state.phase() {
            GamePhase::PickPhase => {
                self.render_pick_phase(state);
            }
            GamePhase::PlacePhase => {
                self.render_place_phase(state);
            }
            GamePhase::EndPhase => {}
        }
    }

    pub fn render_tiles(&mut self, tiles: impl Iterator<Item = Coordinate>, color: Color) {
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

    pub fn render_pick_phase(&mut self, state: &State) {
        for i in 0..N_HEXOS {
            let lower = i * 7;
            let upper = ((i + 1) * 7).min(N_HEXOS);
            let hexos = (lower..upper).map(Hexo::new);
            for (j, hexo) in hexos.enumerate() {
                let x = j as f64 * 100.0 + 30.0;
                let y = i as f64 * 120.0 + 30.0;
                let color = match state.owner_of(hexo) {
                    Some(Player::First) => P1_BLOCK_COLOR,
                    Some(Player::Second) => P2_BLOCK_COLOR,
                    None => DEFAULT_BLOCK_COLOR,
                };
                self.with_affine(Affine::translate((x, y)), |this| {
                    this.render_hexo(hexo, color);
                })
            }
        }
    }

    pub fn render_hexo(&mut self, hexo: Hexo, color: Color) {
        let center = center_of_mass(&hexo) * BLOCK_LENGTH
            - Vec2 {
                x: BLOCK_LENGTH,
                y: BLOCK_LENGTH,
            };
        self.with_affine( Affine::scale(0.6) * Affine::translate(-center) , |this| {
            this.render_tiles(hexo.tiles(), color);
        });
    }

    pub fn render_place_phase(&mut self, state: &State) {
        self.render_board();
    }

    pub fn render_board(&mut self) {
        self.render_board_tiles();
        self.render_tiles(
            Hexo::new(10)
                .apply(Transform::I)
                .move_to(Coordinate::new(5, 7))
                .tiles(),
            P1_BLOCK_COLOR,
        );
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
}
