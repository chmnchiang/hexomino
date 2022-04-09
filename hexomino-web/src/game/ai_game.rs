use std::rc::Rc;

use anyhow::{ensure, Result};
use hexomino_core::{Action, GamePhase, Player};
use itertools::Itertools;
use log::debug;
use rand::prelude::SliceRandom;
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

use super::{CoreGameState, Game, SharedGameState};
use gloo::timers::future::TimeoutFuture;

pub struct AIGame {
    game_state: SharedGameState,
    callback: Callback<()>,
}

impl Game for AIGame {
    fn user_play(self: Rc<Self>, action: Action) -> Result<()> {
        debug!("user action = {:?}", action);
        ensure!(self.user_can_play());

        self.play(action)?;
        Ok(())
    }
    fn user_can_play(&self) -> bool {
        self.game_state.borrow().core_game_state.current_player() == Some(Player::First)
    }
}

impl AIGame {
    pub fn new(game_state: SharedGameState, callback: Callback<()>) -> Self {
        Self {
            game_state,
            callback,
        }
    }

    fn play(self: &Rc<Self>, action: Action) -> Result<()> {
        self.game_state.borrow_mut().core_game_state.play(action)?;
        self.callback.emit(());

        if self.ai_can_play() {
            self.ai_play();
        }
        Ok(())
    }

    fn ai_can_play(&self) -> bool {
        self.game_state.borrow().core_game_state.current_player() == Some(Player::Second)
    }

    fn ai_play(self: &Rc<Self>) {
        debug!("ai play");
        let action = Self::ai_action(&self.game_state.borrow().core_game_state);
        let this = self.clone();
        spawn_local(async move {
            TimeoutFuture::new(200).await;
            this.play(action).unwrap();
        });
    }

    fn ai_action(state: &CoreGameState) -> Action {
        match state.phase() {
            GamePhase::Pick => {
                let remaining_hexos = state.inventory().remaining_hexos().iter().collect_vec();
                let chose_hexo = *remaining_hexos.choose(&mut rand::thread_rng()).unwrap();
                Action::Pick { hexo: chose_hexo }
            }
            GamePhase::Place => {
                let mut ai_hexos = state
                    .inventory()
                    .hexos_of(Player::Second)
                    .iter()
                    .collect_vec();
                ai_hexos.shuffle(&mut rand::thread_rng());
                for hexo in ai_hexos {
                    if let Some(placed_hexo) = state.board().try_find_placement(hexo) {
                        return Action::Place { hexo: placed_hexo };
                    }
                }
                unreachable!("There must be somewhere to place, or else the game should end");
            }
            _ => unreachable!(),
        }
    }
}
