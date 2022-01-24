use piet::{
    kurbo::{Line, Point, Rect},
    Color, RenderContext,
};
use piet_web::{Brush, WebRenderContext};
use web_sys::{CanvasRenderingContext2d, Window};

use crate::game::{
    constants::{COLS, HEXOS, ROWS},
    Hexo, State,
};

pub struct Renderer {
    ctx: WebRenderContext<'static>,
}

const BLOCK_LENGTH: f64 = 30.0;

impl Renderer {
    pub fn new(ctx: CanvasRenderingContext2d, window: Window) -> Self {
        Self {
            ctx: WebRenderContext::new(ctx, window),
        }
    }

    pub fn render_block(&mut self, point: Point) {
        let color = self.ctx.solid_brush(Color::BLACK);
        let fill = self.ctx.solid_brush(Color::rgb8(170, 150, 20));
        self.ctx.fill(
            Rect::new(
                point.x,
                point.y,
                point.x + BLOCK_LENGTH,
                point.y + BLOCK_LENGTH,
            ),
            &fill,
        );
        self.ctx.stroke(
            Rect::new(
                point.x,
                point.y,
                point.x + BLOCK_LENGTH,
                point.y + BLOCK_LENGTH,
            ),
            &color,
            2.0,
        );
    }

    pub fn render_hexo(&mut self, hexo: Hexo, point: Point) {
        for tile in hexo.tiles() {
            self.render_block(
                point
                    + (
                        (tile.x as f64) * BLOCK_LENGTH,
                        (tile.y as f64) * BLOCK_LENGTH,
                    ),
            );
        }
    }

    pub fn render(&mut self, _state: &State) {
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

        self.render_hexo(Hexo::new(0), (60.0, 60.0).into());
        self.render_hexo(Hexo::new(3), (240.0, 60.0).into());

        self.ctx.finish().expect("render failed");
    }
}
