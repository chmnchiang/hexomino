use anyhow::{bail, ensure, Result};
use getset::{CopyGetters, Getters};

use super::{
    board::Board,
    hexo::{Hexo, HexoSet, MovedHexo, PlacedHexo},
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
        use Player::*;
        match self {
            First => Second,
            Second => First,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Getters)]
pub struct Inventory {
    #[getset(get = "pub")]
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
    pub fn hexos_of(&self, player: Player) -> &HexoSet {
        &self.player_hexos[player.id()]
    }
    pub fn owner_of(&self, hexo: Hexo) -> Option<Player> {
        for player in [Player::First, Player::Second] {
            if self.hexos_of(player).has(hexo) {
                return Some(player);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Pick,
    Place,
    End,
}

use GamePhase::*;

#[derive(Getters, CopyGetters, Clone)]
pub struct State {
    #[getset(get_copy = "pub")]
    phase: GamePhase,
    current_player: Player,
    inventory: Inventory,
    board: Board,
}

impl State {
    pub fn current_player(&self) -> Option<Player> {
        match self.phase {
            GamePhase::End => None,
            _ => Some(self.current_player),
        }
    }

    pub fn winner(&self) -> Option<Player> {
        match self.phase {
            GamePhase::End => Some(self.current_player.other()),
            _ => None,
        }
    }

    pub fn inventory(&self) -> &Inventory {
        assert_ne!(self.phase, GamePhase::End);
        &self.inventory
    }

    pub fn board(&self) -> &Board {
        assert_eq!(self.phase, GamePhase::Place);
        &self.board
    }
}

impl State {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            phase: GamePhase::Pick,
            current_player: Player::First,
            inventory: Inventory::new(),
            board: Board::new(),
        }
    }

    pub fn play(&mut self, action: Action) -> Result<()> {
        match (self.phase, action) {
            (GamePhase::Pick, Action::Pick { hexo }) => self.pick(hexo)?,
            (GamePhase::Place, Action::Place { hexo }) => self.place(hexo)?,
            (_, _) => {
                bail!("Action {action:?} is invalid during phase {:?}", self.phase);
            }
        }
        self.next();
        Ok(())
    }

    fn pick(&mut self, hexo: Hexo) -> Result<()> {
        self.inventory.add(self.current_player, hexo)
    }

    fn place(&mut self, hexo: MovedHexo) -> Result<()> {
        ensure!(self.board.can_place(&hexo), "{hexo:?} can not be placed.");
        self.inventory.remove(self.current_player, hexo.hexo())?;
        self.board.place(PlacedHexo::new(hexo, self.current_player))
    }

    fn current_player_can_place(&self) -> bool {
        let hexos = self.inventory.hexos_of(self.current_player);
        if hexos.is_empty() {
            return false;
        }
        for hexo in hexos.iter() {
            if self.board().can_place_somewhere(hexo) {
                return true;
            }
        }
        false
    }

    fn next(&mut self) {
        match self.phase {
            End => {
                panic!("The game had already ended");
            }
            Pick => {
                if self.inventory.remaining_hexos.is_empty() {
                    self.phase = GamePhase::Place;
                    self.current_player = Player::Second;
                } else {
                    self.current_player = self.current_player.other();
                }
            }
            Place => {
                self.current_player = self.current_player.other();
                if !self.current_player_can_place() {
                    self.phase = GamePhase::End;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Pick { hexo: Hexo },
    Place { hexo: MovedHexo },
}

#[cfg(test)]
mod tests {
    use crate::{Transform, Pos};

    use super::*;
    use assert2::{assert, check, let_assert};

    #[test]
    fn after_pick_adds_to_player_set() {
        let mut state = State::new();
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(0) }));
        let_assert!(GamePhase::Pick = state.phase);
        check!(state.inventory.hexos_of(Player::First).has(Hexo::new(0)));
    }

    #[test]
    fn after_pick_next_turn() {
        let mut state = State::new();
        assert!(let Some(Player::First) = state.current_player());
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(0) }));
        check!(state.phase == GamePhase::Pick);
        check!(let Some(Player::Second) = state.current_player());
    }

    #[test]
    fn after_pick_ends_proceeds_to_place() {
        let mut state = State::new();
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(1) }));
        check!(state.phase == GamePhase::Place);
        check!(let Some(Player::Second) = state.current_player());
    }

    #[test]
    fn pick_twice_returns_error() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        check!(let Err(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
    }

    #[test]
    fn player_place_show_on_board() {
        let mut state = State::new();
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(let Ok(_) = state.play(Action::Place {
            hexo: Hexo::new(0).apply(Transform::I).move_to(Pos::ZERO)
        }));
        check!(state.board.is_placed(Pos::new(0, 0)));
        check!(state.board.is_placed(Pos::new(0, 1)));
        check!(state.board.is_placed(Pos::new(0, 2)));
        check!(state.board.is_placed(Pos::new(0, 3)));
        check!(state.board.is_placed(Pos::new(1, 0)));
        check!(state.board.is_placed(Pos::new(1, 1)));
    }

    #[test]
    fn hexo_transform_flip() {
        let mut state = State::new();
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(state.phase == GamePhase::Place);
        assert!(let Ok(_) = state.play(Action::Place {
            hexo: Hexo::new(0)
                .apply(Transform::new(true, 0))
                .move_to(Pos::new(0, 3))
        }));
        check!(state.board.is_placed(Pos::new(0, 0)));
        check!(state.board.is_placed(Pos::new(0, 1)));
        check!(state.board.is_placed(Pos::new(0, 2)));
        check!(state.board.is_placed(Pos::new(0, 3)));
        check!(state.board.is_placed(Pos::new(1, 2)));
        check!(state.board.is_placed(Pos::new(1, 3)));
    }

    #[test]
    fn hexo_transform_rotate() {
        let mut state = State::new();
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(state.phase() == GamePhase::Place);
        assert!(let Ok(_) = state.play(Action::Place {
            hexo: Hexo::new(0)
                .apply(Transform::new(false, 1))
                .move_to(Pos::new(0, 1))
        }));
        check!(state.board.is_placed(Pos::new(0, 0)));
        check!(state.board.is_placed(Pos::new(0, 1)));
        check!(state.board.is_placed(Pos::new(1, 0)));
        check!(state.board.is_placed(Pos::new(1, 1)));
        check!(state.board.is_placed(Pos::new(2, 1)));
        check!(state.board.is_placed(Pos::new(3, 1)));
    }

    #[test]
    fn hexo_transform_flip_rotate() {
        let mut state = State::new();
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(state.phase() == GamePhase::Place);
        assert!(let Ok(_) = state.play(Action::Place {
            hexo: Hexo::new(0)
                .apply(Transform::new(true, 2))
                .move_to(Pos::new(1, 0))
        }));
        check!(state.board.is_placed(Pos::new(0, 0)));
        check!(state.board.is_placed(Pos::new(0, 1)));
        check!(state.board.is_placed(Pos::new(1, 0)));
        check!(state.board.is_placed(Pos::new(1, 1)));
        check!(state.board.is_placed(Pos::new(1, 2)));
        check!(state.board.is_placed(Pos::new(1, 3)));
    }

    #[test]
    fn when_can_not_place_goes_to_end_phase() {
        let mut state = State::new();
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = state.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(state.phase == GamePhase::Place);
        assert!(let Ok(_) = state.play(Action::Place {
            hexo: Hexo::new(0).apply(Transform::I).move_to(Pos::ZERO)
        }));
        assert!(state.phase == GamePhase::End);
        assert!(state.winner() == Some(Player::Second));
    }

    #[test]
    fn when_run_out_of_tiles_goes_to_end_phase() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == GamePhase::Place);
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: Hexo::new(0)
                .apply(Transform::new(false, 1))
                .move_to(Pos::new(0, 2)),
        }));
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: Hexo::new(1).apply(Transform::new(false, 1)).move_to(Pos::ZERO)
        }));
        assert!(game.phase == GamePhase::End);
        assert!(game.winner() == Some(Player::First));
    }

    #[test]
    fn when_can_place_continues() {
        let mut game = State::new();
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(1) }));
        assert!(let Ok(_) = game.play(Action::Pick { hexo: Hexo::new(0) }));
        assert!(game.phase == GamePhase::Place);
        assert!(let Ok(_) = game.play(Action::Place {
            hexo: Hexo::new(0).apply(Transform::new(false, 1)).move_to(Pos::new(0, 3))
        }));
        assert!(game.phase == GamePhase::Place);
        assert!(game.winner().is_none());
    }
}