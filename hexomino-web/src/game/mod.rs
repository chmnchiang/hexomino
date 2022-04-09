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
    pub player_1_name: String,
    pub player_2_name: String,
}

impl PartialEq for GameState {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl GameState {
    fn new(player_1_name: String, player_2_name: String) -> Self {
        Self {
            core_game_state: CoreGameState::new(),
            me: Player::First,
            player_1_name,
            player_2_name,
        }
    }

    pub fn name_of(&self, player: Player) -> &str {
        match player {
            Player::First => &self.player_1_name,
            Player::Second => &self.player_2_name,
        }
    }
}

pub type SharedGameState = Shared<GameState>;

pub trait Game {
    fn user_play(self: Rc<Self>, action: Action) -> Result<()>;
    fn user_can_play(&self) -> bool;
}

pub fn new_game(mode: GameMode, callback: Callback<()>) -> GameBundle {
    match mode {
        GameMode::AI => {
            let game_state = Rc::new(RefCell::new(GameState::new(
                "Player".to_string(),
                "AI".to_string(),
            )));
            GameBundle {
                game: Rc::new(AIGame::new(game_state.clone(), callback)),
                game_state,
            }
        }
        GameMode::TwoPlayer => {
            let game_state = Rc::new(RefCell::new(GameState::new(
                "Player 1".to_string(),
                "Player 2".to_string(),
            )));
            GameBundle {
                game: Rc::new(TwoPlayerGame::new(game_state.clone(), callback)),
                game_state,
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    AI,
    TwoPlayer,
}
