use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use hexomino_core::{Action, Player};
use yew::Callback;

use crate::util::Shared;

use self::{ai_game::AIGame, two_player_game::TwoPlayerGame};

mod ai_game;
mod two_player_game;

pub struct GameBundle {
    pub game: Rc<dyn Game>,
    pub game_state: SharedGameState,
}

pub type CoreGameState = hexomino_core::State;

pub struct GameState {
    pub core_game_state: CoreGameState,
    pub me: Player,
}

impl PartialEq for GameState {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl GameState {
    fn new() -> Self {
        Self {
            core_game_state: CoreGameState::new(),
            me: Player::First,
        }
    }
}

pub type SharedGameState = Shared<GameState>;

pub trait Game {
    fn user_play(self: Rc<Self>, action: Action) -> Result<()>;
    fn user_can_play(&self) -> bool;
}

pub fn new_game(mode: GameMode, callback: Callback<()>) -> GameBundle {
    let game_state = Rc::new(RefCell::new(GameState::new()));
    match mode {
        GameMode::AI => GameBundle {
            game: Rc::new(AIGame::new(game_state.clone(), callback)),
            game_state,
        },
        GameMode::TwoPlayer => GameBundle {
            game: Rc::new(TwoPlayerGame::new(game_state.clone(), callback)),
            game_state,
        },
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    AI,
    TwoPlayer,
}
