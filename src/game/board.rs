use super::{
    constants::{COLS, ROWS},
    hexo::{Hexo, MovedHexo, PlacedHexo},
    pos::Pos,
};
use anyhow::{ensure, Result};
use itertools::Itertools;

#[derive(Debug)]
pub struct Board {
    board: [[bool; COLS]; ROWS],
    placed_hexos: Vec<PlacedHexo>,
}

impl Board {
    pub(super) fn new() -> Self {
        Self {
            board: [[false; COLS]; ROWS],
            placed_hexos: vec![],
        }
    }

    fn in_bound(point: Pos) -> bool {
        0 <= point.x && point.x < ROWS as i32 && 0 <= point.y && point.y < COLS as i32
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
        (0..ROWS)
            .cartesian_product(0..COLS)
            .map(|(x, y)| Pos::new(x as i32, y as i32))
    }

    fn all_empty_tiles<'a>(&'a self) -> impl Iterator<Item = Pos> + 'a {
        self.all_tiles()
            .filter(move |point| !self.is_placed(*point))
    }

    fn can_place(&self, hexo: MovedHexo) -> bool {
        hexo.tiles()
            .all(|tile| Self::in_bound(tile) && !self.is_placed(tile))
    }

    /// Returns true if there is a position on the board the hexo can be placed.
    pub(super) fn can_place_somewhere(&self, hexo: Hexo) -> bool {
        hexo.all_orbit().any(|hexo| {
            self.all_empty_tiles()
                .any(|point| self.can_place(hexo.move_to(point)))
        })
    }

    pub(super) fn place(&mut self, hexo: PlacedHexo) -> Result<()> {
        ensure!(
            self.can_place(hexo.moved_hexo),
            "{hexo:?} can not be placed."
        );
        for point in hexo.moved_hexo.tiles() {
            self.mark_placed(point);
        }
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
                [true, true, true, false, true, true],
                [true, true, true, false, true, true],
                [true, true, false, false, true, true],
                [true, true, false, false, true, true],
            ],
            placed_hexos: vec![],
        };
        check!(board.can_place_somewhere(Hexo::new(0)));
        let board = Board {
            board: [
                [true, true, true, true, true, true],
                [true, true, true, true, true, true],
                [false, false, false, false, true, true],
                [true, true, false, false, true, true],
            ],
            placed_hexos: vec![],
        };
        check!(board.can_place_somewhere(Hexo::new(0)));

        let board = Board {
            board: [
                [false, false, true, false, false, false],
                [true, false, false, false, true, false],
                [false, false, true, false, false, false],
                [true, false, false, false, true, false],
            ],
            placed_hexos: vec![],
        };
        check!(!board.can_place_somewhere(Hexo::new(0)));
    }
}
