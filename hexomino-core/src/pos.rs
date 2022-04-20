use std::ops::Add;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Default, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const ZERO: Self = Pos { x: 0, y: 0 };

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn flip(self) -> Self {
        Self {
            x: -self.x,
            y: self.y,
        }
    }

    /// Rotates the tiles 90 degree clockwise.
    pub fn rotate(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }
}

macro_rules! impl_add_for_point {
    ($ltype:ty, $rtype: ty) => {
        impl Add<$rtype> for $ltype {
            type Output = Pos;
            fn add(self, rhs: $rtype) -> Pos {
                Pos {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                }
            }
        }
    };
}

impl_add_for_point!(Pos, Pos);
impl_add_for_point!(&Pos, Pos);
impl_add_for_point!(Pos, &Pos);
impl_add_for_point!(&Pos, &Pos);

impl From<Pos> for piet::kurbo::Point {
    fn from(point: Pos) -> Self {
        piet::kurbo::Point {
            x: point.x as f64,
            y: point.y as f64,
        }
    }
}

impl From<Pos> for piet::kurbo::Vec2 {
    fn from(point: Pos) -> Self {
        piet::kurbo::Vec2 {
            x: point.x as f64,
            y: point.y as f64,
        }
    }
}
