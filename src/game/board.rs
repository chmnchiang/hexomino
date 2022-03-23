use super::{
    constants::{COLS, ROWS},
    hexo::{Hexo, MovedHexo, PlacedHexo},
    pos::Pos,
};
use anyhow::{ensure, Result};
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Board {
    board: [[bool; ROWS]; COLS],
    placed_hexos: Vec<PlacedHexo>,
}

impl Board {
    pub(super) fn new() -> Self {
        Self {
            board: [[false; ROWS]; COLS],
            placed_hexos: vec![],
        }
    }

    fn in_bound(point: Pos) -> bool {
        0 <= point.x && point.x < COLS as i32 && 0 <= point.y && point.y < ROWS as i32
    }

    pub fn is_placed(&self, point: Pos) -> bool {
        assert!(Self::in_bound(point));
        self.board[point.x as usize][point.y as usize]
    }

    fn mark_placed(&mut self, tile: Pos) {
        assert!(Self::in_bound(tile));
        assert!(!self.is_placed(tile));
        self.board[tile.x as usize][tile.y as usize] = true;
    }

    fn all_tiles(&self) -> impl Iterator<Item = Pos> {
        (0..COLS)
            .cartesian_product(0..ROWS)
            .map(|(x, y)| Pos::new(x as i32, y as i32))
    }

    fn all_empty_tiles(&self) -> impl Iterator<Item = Pos> + '_ {
        self.all_tiles()
            .filter(move |point| !self.is_placed(*point))
    }

    pub fn can_place(&self, hexo: &MovedHexo) -> bool {
        hexo.tiles()
            .all(|tile| Self::in_bound(tile) && !self.is_placed(tile))
    }

    /// Returns true if there is a position on the board the hexo can be placed.
    pub fn try_find_placement(&self, hexo: Hexo) -> Option<MovedHexo> {
        for rhexo in hexo.all_orbit() {
            for pos in self.all_empty_tiles() {
                let moved_hexo = rhexo.move_to(pos);
                if self.can_place(&moved_hexo) {
                    return Some(moved_hexo);
                }
            }
        }
        None
    }

    /// Returns true if there is a position on the board the hexo can be placed.
    pub fn can_place_somewhere(&self, hexo: Hexo) -> bool {
        self.try_find_placement(hexo).is_some()
    }

    pub(super) fn place(&mut self, hexo: PlacedHexo) -> Result<()> {
        ensure!(
            self.can_place(hexo.moved_hexo()),
            "{hexo:?} can not be placed."
        );
        for point in hexo.moved_hexo().tiles() {
            self.mark_placed(point);
        }
        self.placed_hexos.push(hexo);
        Ok(())
    }

    pub fn placed_hexos(&self) -> &[PlacedHexo] {
        &self.placed_hexos
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use assert2::check;
    #[test]
    fn board_can_place_in_any() {
        let board = Board {
            board: [
                [true, true, true, true],
                [true, true, true, true],
                [true, true, false, false],
                [false, false, false, false],
                [true, true, true, true],
                [true, true, true, true],
            ],
            placed_hexos: vec![],
        };
        check!(board.can_place_somewhere(Hexo::new(0)));
        let board = Board {
            board: [
                [true, true, false, true],
                [true, true, false, true],
                [true, true, false, false],
                [true, true, false, false],
                [true, true, true, true],
                [true, true, true, true],
            ],
            placed_hexos: vec![],
        };
        check!(board.can_place_somewhere(Hexo::new(0)));

        let board = Board {
            board: [
                [false, true, false, true],
                [false, false, false, false],
                [true, false, true, false],
                [false, false, false, false],
                [false, true, false, true],
                [false, false, false, false],
            ],
            placed_hexos: vec![],
        };
        check!(!board.can_place_somewhere(Hexo::new(0)));
    }
}
