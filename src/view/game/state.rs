use std::{cell::RefCell, rc::Rc};

use crate::game::state::Player;

pub type GameState = crate::game::state::State;

pub struct GameViewState {
    pub game_state: GameState,
    pub me: Player,
}

impl PartialEq for GameViewState {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub type SharedGameViewState = Rc<RefCell<GameViewState>>;
