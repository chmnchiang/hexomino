use crate::Tiles;
use crate::Pos;

pub struct __Hexo {
    pub tiles: Tiles,
}

#[cfg(not(test))]
pub const ROWS: usize = 12;
#[cfg(not(test))]
pub const COLS: usize = 18;

include!(concat!(env!("OUT_DIR"), "/hexos.rs"));

#[cfg(test)]
pub const ROWS: usize = 4;
#[cfg(test)]
pub const COLS: usize = 6;
#[cfg(test)]
pub const N_HEXOS: usize = 2;
#[cfg(test)]
pub const HEXOS: [__Hexo; 2] = [
    __Hexo {
        tiles: [
            Pos { x: 0, y: 0 },
            Pos { x: 0, y: 1 },
            Pos { x: 0, y: 2 },
            Pos { x: 0, y: 3 },
            Pos { x: 1, y: 0 },
            Pos { x: 1, y: 1 },
        ],
    },
    __Hexo {
        tiles: [
            Pos { x: 0, y: 0 },
            Pos { x: 0, y: 1 },
            Pos { x: 0, y: 2 },
            Pos { x: 0, y: 3 },
            Pos { x: 0, y: 4 },
            Pos { x: 0, y: 5 },
        ],
    },
];
