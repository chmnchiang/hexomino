use std::{cell::RefCell, rc::Rc};

use api::GameEndState;
use gloo::timers::future::TimeoutFuture;
use hexomino_core::{Action, GamePhase, Player};
use itertools::Itertools;
use rand::prelude::SliceRandom;
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html};

use crate::game::{GameState, SharedGameState};

use super::{
    end_view::EndView, pick_view::PickView, place_view::PlaceView, turn_indicator::TurnIndicator,
};

pub struct AiGameView {
    game: SharedGameState,
    my_score: u32,
    ai_score: u32,
}

pub enum AiGameMsg {
    UserPlay(Action),
    AiPlay(Action),
    Restart,
}

impl Component for AiGameView {
    type Message = AiGameMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            game: Rc::new(RefCell::new(GameState::new(Player::First))),
            my_score: 0,
            ai_score: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: AiGameMsg) -> bool {
        use AiGameMsg::*;
        match msg {
            UserPlay(action) => self.user_play(action, ctx),
            AiPlay(action) => self.ai_play(action),
            Restart => self.restart_game(),
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let game_state = self.game.borrow();
        let me = game_state.me();
        let core_game_state = game_state.core();

        let mut player_names = ["You", "AI"].map(|s| s.to_string());
        let mut scores = [self.my_score, self.ai_score];
        if me == Player::Second {
            player_names.swap(0, 1);
            scores.swap(0, 1);
        }
        let send_pick = ctx
            .link()
            .callback(|hexo| AiGameMsg::UserPlay(Action::Pick(hexo)));
        let send_place = ctx
            .link()
            .callback(|moved_hexo| AiGameMsg::UserPlay(Action::Place(moved_hexo)));
        let play_again_onclick = ctx.link().callback(|_| AiGameMsg::Restart);

        html! {
            <div>
                <TurnIndicator {me}
                    current_player={core_game_state.current_player()}
                    player_names={player_names.clone()}
                    {scores}/>
                {
                    match core_game_state.phase() {
                        GamePhase::Pick => html!{
                            <PickView state={self.game.clone()} send_pick={send_pick}/>
                        },
                        GamePhase::Place => html!{
                            <PlaceView state={self.game.clone()} send_place={send_place}/>
                        },
                        GamePhase::End => html!{
                            <>
                                <EndView state={self.game.clone()} names={player_names}/>
                                <div class="columns is-centered">
                                    <div class="column is-one-quarter" style="text-align: center">
                                        <button class="button is-success" onclick={play_again_onclick}>
                                            <span class="icon">
                                                <i class="fa-solid fa-arrow-rotate-left"></i>
                                            </span>
                                            <span>{"Play again"}</span>
                                        </button>
                                    </div>
                                </div>
                            </>
                        },
                    }
                }
            </div>
        }
    }
}

impl AiGameView {
    fn user_play(&mut self, action: Action, ctx: &Context<Self>) {
        let game_clone = self.game.clone();
        {
            let game_state = self.game.borrow_mut();
            if game_state.core().current_player() != Some(game_state.me()) {
                return;
            }
        }
        self.current_player_play(action);
        let ai_play_callback = ctx.link().callback(AiGameMsg::AiPlay);

        spawn_local(async move {
            while let Some(ai_next_action) = {
                let game_state = game_clone.borrow();
                calculate_ai_action(&*game_state)
            } {
                TimeoutFuture::new(500).await;
                ai_play_callback.emit(ai_next_action);
            }
        })
    }

    fn ai_play(&mut self, action: Action) {
        self.current_player_play(action);
    }

    fn restart_game(&mut self) {
        let game_idx = self.my_score + self.ai_score;
        self.game = Rc::new(RefCell::new(GameState::new(if game_idx % 2 == 0 {
            Player::First
        } else {
            Player::Second
        })));

        if let Some(ai_next_action) = {
            let game_state = self.game.borrow();
            calculate_ai_action(&*game_state)
        } {
            self.current_player_play(ai_next_action);
        }
    }

    fn current_player_play(&mut self, action: Action) {
        let mut game_state = self.game.borrow_mut();
        let _ = game_state.current_player_play(action);
        if let Some(winner) = game_state.core().winner() {
            if winner == game_state.me() {
                self.my_score += 1;
            } else {
                self.ai_score += 1;
            }
            game_state.set_end_state(GameEndState {
                winner,
                reason: api::GameEndReason::NoValidMove,
            })
        }
    }
}

fn calculate_ai_action(game_state: &GameState) -> Option<Action> {
    if !game_state
        .core()
        .current_player()
        .is_some_and(|p| *p != game_state.me())
    {
        return None;
    }
    let core_state = game_state.core();
    match core_state.phase() {
        GamePhase::Pick => {
            let remaining_hexos = core_state
                .inventory()
                .remaining_hexos()
                .iter()
                .collect_vec();
            let chose_hexo = *remaining_hexos.choose(&mut rand::thread_rng()).unwrap();
            Some(Action::Pick(chose_hexo))
        }
        GamePhase::Place => {
            let mut ai_hexos = core_state
                .inventory()
                .hexos_of(game_state.me().other())
                .iter()
                .collect_vec();
            ai_hexos.shuffle(&mut rand::thread_rng());
            for hexo in ai_hexos {
                if let Some(placed_hexo) = core_state.board().try_find_placement(hexo) {
                    return Some(Action::Place(placed_hexo));
                }
            }
            unreachable!("There must be somewhere to place, or else the game should end");
        }
        _ => None,
    }
}
