use std::rc::Rc;

use anyhow::Result;
use hexomino_core::Action;
use yew::Callback;

use super::{SharedGameState, Game};


pub struct TwoPlayerGame {
    game_state: SharedGameState,
    callback: Callback<()>,
}

impl Game for TwoPlayerGame {
    fn user_play(self: Rc<Self>, action: Action) -> Result<()> {
        self.game_state.borrow_mut().core_game_state.play(action)?;
        self.callback.emit(());
        Ok(())
    }
    fn user_can_play(&self) -> bool {
        true
    }
}

impl TwoPlayerGame {
    pub fn new(game_state: SharedGameState, callback: Callback<()>) -> Self {
        Self {
            game_state,
            callback,
        }
    }
}
