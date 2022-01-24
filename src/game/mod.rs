use anyhow::{bail, ensure, Result};
use itertools::Itertools;

pub mod constants;
pub mod hexo;
pub mod point;

use crate::game::hexo::Transform;

use self::{
    constants::{COLS, ROWS},
    hexo::{HexoSet, MovedHexo},
};

pub use self::hexo::Hexo;
pub use self::point::Point;

//type Board = [[bool; COLS]; ROWS];

#[derive(Debug)]
pub struct Board {
    board: [[bool; COLS]; ROWS],
}

pub struct PlacedHexo {
    hexo: Hexo,
    pos: Point,
    player: Player,
}

impl Board {
    fn new() -> Self {
        Self {
            board: [[false; COLS]; ROWS],
        }
    }

    fn in_bound(point: Point) -> bool {
        0 <= point.x && point.x < ROWS as i32 && 0 <= point.y && point.y < COLS as i32
    }

    fn is_placed(&self, point: Point) -> bool {
        assert!(Self::in_bound(point));
        self.board[point.x as usize][point.y as usize]
    }

    fn place_tile(&mut self, tile: Point) {
        assert!(Self::in_bound(tile));
        assert!(!self.is_placed(tile));
        self.board[tile.x as usize][tile.y as usize] = true;
    }

    fn all_tiles(&self) -> impl Iterator<Item = Point> {
        (0..ROWS)
            .cartesian_product(0..COLS)
            .map(|(x, y)| Point::new(x as i32, y as i32))
    }

    fn all_empty_tiles<'a>(&'a self) -> impl Iterator<Item = Point> + 'a {
        self.all_tiles()
            .filter(move |point| !self.is_placed(*point))
    }

    fn can_place(&self, hexo: MovedHexo) -> bool {
        hexo.tiles()
            .all(|tile| Self::in_bound(tile) && !self.is_placed(tile))
    }

    /// Returns true if there is a position on the board the hexo can be placed.
    fn can_place_somewhere(&self, hexo: Hexo) -> bool {
        hexo.all_orbit().any(|hexo| {
            self.all_empty_tiles()
                .any(|point| self.can_place(hexo.move_to(point)))
        })
    }

    fn place(&mut self, hexo: MovedHexo) -> Result<()> {
        ensure!(self.can_place(hexo), "{hexo:?} can not be placed.");
        for point in hexo.tiles() {
            self.place_tile(point);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GamePhase {
    PickPhase,
    PlacePhase,
    EndPhase,
}

use GamePhase::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    First = 0,
    Second = 1,
}

impl Player {
    fn id(self) -> usize {
        self as usize
    }
    fn other(self) -> Player {
        match self {
            First => Second,
            Second => First,
        }
    }
}

use Player::*;

#[derive(Debug)]
pub struct State {
    phase: GamePhase,
    board: Board,
    turn: Player,
    remaining_hexos: HexoSet,
    player_hexos: [HexoSet; 2],
    winner: Option<Player>,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Pick { hexo: Hexo },
    Place { hexo: MovedHexo },
}

impl State {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            phase: GamePhase::PickPhase,
            turn: Player::First,
            remaining_hexos: HexoSet::all(),
            player_hexos: [HexoSet::empty(), HexoSet::empty()],
            winner: None,
        }
    }

    pub fn play(&mut self, action: Action) -> Result<()> {
        match (self.phase, action) {
            (PickPhase, Action::Pick { hexo }) => self.pick(hexo),
            (PlacePhase, Action::Place { hexo }) => self.place(hexo),
            _ => {
                bail!("Action {action:?} is invalid during phase {:?}", self.phase)
            }
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn current_player(&self) -> Player {
        self.turn
    }

    fn current_player_can_place(&self) -> bool {
        let hexos = &self.player_hexos[self.current_player().id()];
        if hexos.is_empty() {
            return false;
        }
        for hexo in hexos.iter() {
            if self.board.can_place_somewhere(hexo) {
                return true;
            }
        }
        false
    }

    fn next(&mut self) {
        match self.phase {
            EndPhase => {
                panic!("The game had already ended");
            }
            PickPhase => {
                if self.remaining_hexos.is_empty() {
                    self.phase = PlacePhase;
                    self.turn = Second;
                } else {
                    self.turn = self.turn.other();
                }
            }
            PlacePhase => {
                self.turn = self.turn.other();
                if !self.current_player_can_place() {
                    self.phase = EndPhase;
                    self.winner = Some(self.turn.other());
                }
            }
        }
    }

    fn pick(&mut self, hexo: Hexo) -> Result<()> {
        assert!(self.phase == PickPhase);
        ensure!(
            self.remaining_hexos.has(hexo),
            "{hexo:?} has been picked, but player {:?} tries to pick it.",
            self.current_player()
        );
        self.remaining_hexos.remove(hexo);
        self.player_hexos[self.current_player().id()].add(hexo);
        self.next();
        Ok(())
    }

    fn place(&mut self, hexo: MovedHexo) -> Result<()> {
        assert!(self.phase == PlacePhase);
        let current_player_hexos = &mut self.player_hexos[self.current_player().id()];
        ensure!(
            current_player_hexos.has(hexo.hexo()),
            "Player {:?} tries to play {hexo:?} but does not have it.",
            self.current_player()
        );
        self.board.place(hexo)?;
        current_player_hexos.remove(hexo.hexo());
        self.next();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::game::hexo::Transform;

    use super::*;
    use assert2::{assert, check};

    #[test]
    fn cfg_test_is_working() {
        check!(super::constants::N_HEXOS == 2);
    }

    #[test]
    fn after_pick_adds_to_player_set() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        check!(game.player_hexos[0].has(Hexo::new(0)));
    }

    #[test]
    fn after_pick_next_turn() {
        let mut game = State::new();
        assert!(game.current_player() == First);
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        check!(game.current_player() == Second);
    }

    #[test]
    fn after_pick_ends_proceeds_to_place() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        check!(game.phase == PlacePhase);
        check!(game.turn == Second);
    }

    #[test]
    fn pick_twice_returns_error() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        check!(let Err(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
    }

    #[test]
    fn player_place_show_on_board() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == PlacePhase);
        assert!(let Ok(_) = game.play(Action::Place { hexo: MovedHexo::new(Hexo::new(0), Default::default()) }));
        check!(game.board.is_placed(Point::new(0, 0)));
        check!(game.board.is_placed(Point::new(0, 1)));
        check!(game.board.is_placed(Point::new(0, 2)));
        check!(game.board.is_placed(Point::new(0, 3)));
        check!(game.board.is_placed(Point::new(1, 0)));
        check!(game.board.is_placed(Point::new(1, 1)));
    }

    #[test]
    fn hexo_transform_flip() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == PlacePhase);
        let transform = Transform::new(true, 0, Point::new(0, 3));
        assert!(let Ok(_) = game.play(Action::Place { hexo: MovedHexo::new(Hexo::new(0), transform) }));
        check!(game.board.is_placed(Point::new(0, 0)));
        check!(game.board.is_placed(Point::new(0, 1)));
        check!(game.board.is_placed(Point::new(0, 2)));
        check!(game.board.is_placed(Point::new(0, 3)));
        check!(game.board.is_placed(Point::new(1, 2)));
        check!(game.board.is_placed(Point::new(1, 3)));
    }

    #[test]
    fn hexo_transform_rotate() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == PlacePhase);
        let transform = Transform::new(false, 1, Point::new(0, 1));
        assert!(let Ok(_) = game.play(Action::Place { hexo: MovedHexo::new(Hexo::new(0), transform) }));
        check!(game.board.is_placed(Point::new(0, 0)));
        check!(game.board.is_placed(Point::new(0, 1)));
        check!(game.board.is_placed(Point::new(1, 0)));
        check!(game.board.is_placed(Point::new(1, 1)));
        check!(game.board.is_placed(Point::new(2, 1)));
        check!(game.board.is_placed(Point::new(3, 1)));
    }

    #[test]
    fn hexo_transform_flip_rotate() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == PlacePhase);
        let transform = Transform::new(true, 2, Point::new(1, 0));
        assert!(let Ok(_) = game.play(Action::Place { hexo: MovedHexo::new(Hexo::new(0), transform) }));
        check!(game.board.is_placed(Point::new(0, 0)));
        check!(game.board.is_placed(Point::new(0, 1)));
        check!(game.board.is_placed(Point::new(1, 0)));
        check!(game.board.is_placed(Point::new(1, 1)));
        check!(game.board.is_placed(Point::new(1, 2)));
        check!(game.board.is_placed(Point::new(1, 3)));
    }

    #[test]
    fn board_can_place_in_any() {
        let board = Board {
            board: [
                [true, true, true, false, true, true],
                [true, true, true, false, true, true],
                [true, true, false, false, true, true],
                [true, true, false, false, true, true],
            ],
        };
        check!(board.can_place_somewhere(Hexo::new(0)));
        let board = Board {
            board: [
                [true, true, true, true, true, true],
                [true, true, true, true, true, true],
                [false, false, false, false, true, true],
                [true, true, false, false, true, true],
            ],
        };
        check!(board.can_place_somewhere(Hexo::new(0)));

        let board = Board {
            board: [
                [false, false, true, false, false, false],
                [true, false, false, false, true, false],
                [false, false, true, false, false, false],
                [true, false, false, false, true, false],
            ],
        };
        check!(!board.can_place_somewhere(Hexo::new(0)));
    }

    #[test]
    fn when_can_not_place_goes_to_end_phase() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == PlacePhase);
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: MovedHexo::new(Hexo::new(0), Transform::new(false, 1, Point::new(0, 3)))
        }));
        assert!(let EndPhase = game.phase);
        assert!(game.winner == Some(Second));
    }

    #[test]
    fn when_run_out_of_tiles_goes_to_end_phase() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == PlacePhase);
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: MovedHexo::new(Hexo::new(0), Default::default())
        }));
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: MovedHexo::new(Hexo::new(1), Transform::new(false, 0, Point::new(2, 0)))
        }));
        assert!(let EndPhase = game.phase);
        assert!(game.winner == Some(First));
    }

    #[test]
    fn when_can_place_continues() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == PlacePhase);
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: MovedHexo::new(Hexo::new(0), Default::default())
        }));
        assert!(game.phase == PlacePhase);
    }
}
