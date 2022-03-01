use std::mem;

use anyhow::{bail, ensure, Result};

use super::{
    board::Board,
    constants::{COLS, ROWS},
    hexo::{Hexo, HexoSet, MovedHexo, PlacedHexo},
    pos::Pos,
};

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

struct Inventory {
    remaining_hexos: HexoSet,
    player_hexos: [HexoSet; 2],
}

impl Inventory {
    fn new() -> Self {
        Self {
            remaining_hexos: HexoSet::all(),
            player_hexos: [HexoSet::empty(), HexoSet::empty()],
        }
    }
    fn add(&mut self, player: Player, hexo: Hexo) -> Result<()> {
        ensure!(
            self.remaining_hexos.has(hexo),
            "{hexo:?} has been picked, but player {:?} tries to pick it.",
            player
        );
        self.remaining_hexos.remove(hexo);
        self.player_hexos[player.id()].add(hexo);
        Ok(())
    }
    fn remove(&mut self, player: Player, hexo: Hexo) -> Result<()> {
        let current_player_hexos = &mut self.player_hexos[player.id()];
        ensure!(
            current_player_hexos.has(hexo),
            "Player {:?} tries to play {hexo:?}, but they do not have it.",
            player,
        );
        current_player_hexos.remove(hexo);
        Ok(())
    }
    fn hexos_of(&self, player: Player) -> &HexoSet {
        &self.player_hexos[player.id()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    PickPhase,
    PlacePhase,
    EndPhase,
}

use GamePhase::*;

pub enum _State {
    Pick(PickState),
    Place(PlaceState),
    End(EndState),
    _Moved,
}

impl _State {
    pub fn new() -> Self {
        _State::Pick(PickState {
            current_player: First,
            inventory: Inventory::new(),
        })
    }
    pub fn phase(&self) -> GamePhase {
        match self {
            _State::Pick(..) => PickPhase,
            _State::Place(..) => PlacePhase,
            _State::End(..) => EndPhase,
            _State::_Moved => unreachable!(),
        }
    }
    pub fn current_player(&self) -> Option<Player> {
        match self {
            _State::Pick(state) => Some(state.current_player),
            _State::Place(state) => Some(state.current_player),
            _State::End(..) => None,
            _State::_Moved => unreachable!(),
        }
    }
    pub fn play(&mut self, action: Action) -> Result<()> {
        match (&mut *self, action) {
            (_State::_Moved, _) => unreachable!(),
            (_State::Pick(state), Action::Pick { hexo }) => {
                state.pick(hexo)?;
            }
            (_State::Place(state), Action::Place { hexo }) => {
                state.place(hexo)?;
            }
            _ => {
                bail!("Action {action:?} is invalid during phase {:?}", self.phase())
            }
        }
        self.next();
        Ok(())
    }
    fn next(&mut self) {
        use _State::*;
        let state = mem::replace(self, _Moved);
        *self = match state {
            End(..) => {
                panic!("The game had already ended");
            }
            Pick(state) => {
                if state.inventory.remaining_hexos.is_empty() {
                    Place(PlaceState {
                        current_player: Second,
                        inventory: state.inventory,
                        board: Board::new(),
                    })
                } else {
                    Pick(PickState {
                        current_player: state.current_player.other(),
                        inventory: state.inventory,
                    })
                }
            }
            Place(mut state) => {
                state.current_player = state.current_player.other();
                if !state.current_player_can_place() {
                    End(EndState {
                        winner: state.current_player.other(),
                        board: state.board,
                    })
                } else {
                    Place(state)
                }
            }
            _Moved => unreachable!(),
        }
    }
}

pub struct PickState {
    current_player: Player,
    inventory: Inventory,
}

impl PickState {
    fn current_player(&self) -> Player {
        self.current_player
    }
    fn pick(&mut self, hexo: Hexo) -> Result<()> {
        self.inventory.add(self.current_player, hexo)
    }
}

pub struct PlaceState {
    current_player: Player,
    inventory: Inventory,
    board: Board,
}

impl PlaceState {
    fn current_player(&self) -> Player {
        self.current_player
    }
    fn place(&mut self, hexo: MovedHexo) -> Result<()> {
        self.board.place(PlacedHexo {
            moved_hexo: hexo,
            player: self.current_player,
        })?;
        self.inventory.remove(self.current_player, hexo.hexo())?;
        Ok(())
    }
    fn current_player_can_place(&self) -> bool {
        let hexos = self.inventory.hexos_of(self.current_player);
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
}

pub struct EndState {
    winner: Player,
    board: Board,
}

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

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn current_player(&self) -> Player {
        self.turn
    }

    pub fn phase(&self) -> GamePhase {
        self.phase
    }

    pub fn owner_of(&self, hexo: Hexo) -> Option<Player> {
        if self.player_hexos[0].has(hexo) {
            Some(First)
        } else if self.player_hexos[1].has(hexo) {
            Some(Second)
        } else {
            None
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
        let current_player = self.current_player();
        let current_player_hexos = &mut self.player_hexos[self.current_player().id()];
        ensure!(
            current_player_hexos.has(hexo.hexo()),
            "Player {:?} tries to play {hexo:?} but does not have it.",
            self.current_player()
        );
        self.board.place(PlacedHexo {
            moved_hexo: hexo,
            player: current_player,
        })?;
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
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: Hexo::new(0).apply(Transform::I).move_to(Pos::ZERO)
        }));
        check!(game.board.is_placed(Pos::new(0, 0)));
        check!(game.board.is_placed(Pos::new(0, 1)));
        check!(game.board.is_placed(Pos::new(0, 2)));
        check!(game.board.is_placed(Pos::new(0, 3)));
        check!(game.board.is_placed(Pos::new(1, 0)));
        check!(game.board.is_placed(Pos::new(1, 1)));
    }

    #[test]
    fn hexo_transform_flip() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == PlacePhase);
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: Hexo::new(0)
                .apply(Transform::new(true, 0))
                .move_to(Pos::new(0, 3))
        }));
        check!(game.board.is_placed(Pos::new(0, 0)));
        check!(game.board.is_placed(Pos::new(0, 1)));
        check!(game.board.is_placed(Pos::new(0, 2)));
        check!(game.board.is_placed(Pos::new(0, 3)));
        check!(game.board.is_placed(Pos::new(1, 2)));
        check!(game.board.is_placed(Pos::new(1, 3)));
    }

    #[test]
    fn hexo_transform_rotate() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == PlacePhase);
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: Hexo::new(0)
                .apply(Transform::new(false, 1))
                .move_to(Pos::new(0, 1))
        }));
        check!(game.board.is_placed(Pos::new(0, 0)));
        check!(game.board.is_placed(Pos::new(0, 1)));
        check!(game.board.is_placed(Pos::new(1, 0)));
        check!(game.board.is_placed(Pos::new(1, 1)));
        check!(game.board.is_placed(Pos::new(2, 1)));
        check!(game.board.is_placed(Pos::new(3, 1)));
    }

    #[test]
    fn hexo_transform_flip_rotate() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == PlacePhase);
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: Hexo::new(0)
                .apply(Transform::new(true, 2))
                .move_to(Pos::new(1, 0))
        }));
        check!(game.board.is_placed(Pos::new(0, 0)));
        check!(game.board.is_placed(Pos::new(0, 1)));
        check!(game.board.is_placed(Pos::new(1, 0)));
        check!(game.board.is_placed(Pos::new(1, 1)));
        check!(game.board.is_placed(Pos::new(1, 2)));
        check!(game.board.is_placed(Pos::new(1, 3)));
    }

    #[test]
    fn when_can_not_place_goes_to_end_phase() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == PlacePhase);
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: Hexo::new(0)
                .apply(Transform::new(false, 1))
                .move_to(Pos::new(0, 3))
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
            hexo: Hexo::new(0)
                .apply(Transform::I)
                .move_to(Pos::ZERO),
        }));
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: Hexo::new(1)
                .apply(Transform::new(false, 0))
                .move_to(Pos::new(2, 0))
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
            hexo: Hexo::new(0).apply(Transform::I).move_to(Pos::ZERO)
        }));
        assert!(game.phase == PlacePhase);
    }
}
