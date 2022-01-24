use std::ops::Add;

#[derive(Clone, Copy, Eq, PartialEq, Default, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn flip(self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
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
            type Output = Point;
            fn add(self, rhs: $rtype) -> Point {
                Point {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                }
            }
        }
    };
}

impl_add_for_point!(Point, Point);
impl_add_for_point!(&Point, Point);
impl_add_for_point!(Point, &Point);
impl_add_for_point!(&Point, &Point);
