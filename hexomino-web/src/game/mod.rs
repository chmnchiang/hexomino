use std::{cell::RefCell, rc::Rc};

use api::{
    GameEndInfo, GameEndReason, GameEndState, MatchEndInfo, MatchInfo, MatchWinner, UserPlay,
};
use getset::{CopyGetters, Getters};
use hexomino_core::{Action, Player};
use yew::Callback;

use crate::util::Shared;

#[derive(thiserror::Error, Debug)]
pub enum MatchError {
    #[error("state is not synced")]
    StateNotSynced,
    #[error("game error: {0}")]
    GameError(#[from] hexomino_core::Error),
}

type Result<T> = std::result::Result<T, MatchError>;

#[derive(Getters)]
pub struct MatchState {
    info: MatchInfo,
    game_idx: i32,
    #[getset(get = "pub")]
    scores: [u32; 2],
    #[getset(get = "pub")]
    state: MatchInnerState,
    //#[getset(get = "pub")]
    //game: Option<SharedGameState>,
    //#[getset(get = "pub")]
    //match_end_info: Option<MatchEndInfo>,
}

pub enum MatchInnerState {
    NotStarted,
    Playing(SharedGameState),
    Ended { winner: MatchWinner },
}

#[derive(PartialEq, Eq)]
pub enum MatchPhase {
    MatchNotStarted,
    GamePlaying,
    GameEnded,
    MatchEnded,
}

pub type SharedGameState = Shared<GameState>;
#[derive(Getters, CopyGetters)]
pub struct GameState {
    #[getset(get = "pub")]
    core: hexomino_core::State,
    #[getset(get_copy = "pub")]
    me: Player,
    num_action: usize,
    #[getset(get = "pub")]
    end_state: Option<api::GameEndState>,
}

impl MatchState {
    pub fn from_api(match_state: api::MatchState) -> Self {
        let inner_state = match match_state.state {
            api::MatchInnerState::NotStarted => MatchInnerState::NotStarted,
            api::MatchInnerState::Playing(game_state) => match game_state {
                api::GameState::GamePlaying(api_state) => MatchInnerState::Playing(Rc::new(
                    RefCell::new(GameState::new_from_api(api_state)),
                )),
                api::GameState::GameEnded {
                    game_state: api_state,
                    end_state,
                } => {
                    let mut game_state = GameState::new_from_api(api_state);
                    game_state.end_state = Some(end_state);
                    MatchInnerState::Playing(Rc::new(RefCell::new(game_state)))
                }
            },
            api::MatchInnerState::Ended { winner } => MatchInnerState::Ended { winner },
        };

        MatchState {
            info: match_state.info,
            game_idx: match_state.game_idx,
            scores: match_state.scores,
            state: inner_state,
        }
    }

    pub fn phase(&self) -> MatchPhase {
        match &self.state {
            MatchInnerState::NotStarted => MatchPhase::MatchNotStarted,
            MatchInnerState::Playing(game_state) => match game_state.borrow().end_state() {
                None => MatchPhase::GamePlaying,
                Some(_) => MatchPhase::GameEnded,
            },
            MatchInnerState::Ended { .. } => MatchPhase::MatchEnded,
        }
    }

    pub fn player_name(&self, idx: usize) -> &str {
        &self.info.user_data[idx].name
    }

    pub fn update_new_game(&mut self, me: Player) -> Result<()> {
        if self.phase() != MatchPhase::MatchNotStarted && self.phase() != MatchPhase::GameEnded {
            return Err(MatchError::StateNotSynced);
        }
        self.state = MatchInnerState::Playing(Rc::new(RefCell::new(GameState::new(me))));
        Ok(())
    }

    pub fn update_action(&mut self, UserPlay { idx, action }: UserPlay) -> Result<()> {
        let MatchInnerState::Playing(game_state) = &self.state else {
            return Err(MatchError::StateNotSynced);
        };
        let mut game_state = game_state.borrow_mut();
        if game_state.num_action != idx as usize {
            return Err(MatchError::StateNotSynced);
        }
        game_state.num_action += 1;
        Ok(game_state.core.current_player_play(action)?)
    }

    pub fn update_game_end(&mut self, info: GameEndInfo) -> Result<()> {
        if self.phase() != MatchPhase::GamePlaying {
            return Err(MatchError::StateNotSynced);
        }
        self.scores = info.scores;
        let MatchInnerState::Playing(game_state) = &self.state else {
            return Err(MatchError::StateNotSynced);
        };
        game_state.borrow_mut().end_state = Some(info.end_state);
        Ok(())
    }

    pub fn update_match_end(&mut self, info: MatchEndInfo) -> Result<()> {
        self.scores = info.scores;
        self.state = MatchInnerState::Ended {
            winner: info.winner,
        };
        Ok(())
    }

    pub fn names(&self) -> [String; 2] {
        [0, 1].map(|idx| self.info.user_data[idx].name.clone())
    }

    pub fn scores_ord_by_player(&self) -> [u32; 2] {
        let MatchInnerState::Playing(game_state) = &self.state else {
            panic!("game is not started");
        };
        let me = game_state.borrow().me();
        match me {
            Player::First => self.scores,
            Player::Second => [self.scores[1], self.scores[0]],
        }
    }

    pub fn names_ord_by_player(&self) -> [String; 2] {
        let MatchInnerState::Playing(game_state) = &self.state else {
            panic!("game is not started");
        };
        let me = game_state.borrow().me();
        let indices = match me {
            Player::First => [0, 1],
            Player::Second => [1, 0],
        };
        indices.map(|idx| self.info.user_data[idx].name.clone())
    }
}

impl GameState {
    fn new(me: Player) -> Self {
        Self {
            core: hexomino_core::State::new(),
            me,
            num_action: 0,
            end_state: None,
        }
    }
    fn new_from_api(state: api::GameInnerState) -> Self {
        let mut game = GameState::new(state.you);
        for action in state.prev_actions {
            let _ = game.core.current_player_play(action);
        }
        game
    }
}

impl PartialEq for GameState {
    fn eq(&self, other: &Self) -> bool {
        self.num_action == other.num_action
    }
}

//use self::{ai_game::AIGame, two_player_game::TwoPlayerGame};

//mod ai_game;
//mod two_player_game;

//pub struct GameBundle {
//pub game: Rc<dyn Game>,
//pub game_state: SharedGameState,
//}

//pub type CoreGameState = hexomino_core::State;

//pub struct GameState {
//pub core_game_state: CoreGameState,
//pub me: Player,
//pub player_1_name: String,
//pub player_2_name: String,
//}

//impl PartialEq for GameState {
//fn eq(&self, _other: &Self) -> bool {
//false
//}
//}

//impl GameState {
//pub fn new(player_1_name: String, player_2_name: String) -> Self {
//Self {
//core_game_state: CoreGameState::new(),
//me: Player::First,
//player_1_name,
//player_2_name,
//}
//}

//pub fn name_of(&self, player: Player) -> &str {
//match player {
//Player::First => &self.player_1_name,
//Player::Second => &self.player_2_name,
//}
//}
//}

//pub type SharedGameState = Shared<GameState>;

//pub trait Game {
//fn user_play(self: Rc<Self>, action: Action) -> Result<()>;
//fn user_can_play(&self) -> bool;
//}

//pub fn new_game(mode: GameMode, callback: Callback<()>) -> GameBundle {
//match mode {
//GameMode::AI => {
//let game_state = Rc::new(RefCell::new(GameState::new(
//"Player".to_string(),
//"AI".to_string(),
//)));
//GameBundle {
//game: Rc::new(AIGame::new(game_state.clone(), callback)),
//game_state,
//}
//}
//GameMode::TwoPlayer => {
//let game_state = Rc::new(RefCell::new(GameState::new(
//"Player 1".to_string(),
//"Player 2".to_string(),
//)));
//GameBundle {
//game: Rc::new(TwoPlayerGame::new(game_state.clone(), callback)),
//game_state,
//}
//}
//}
//}

//#[derive(Clone, Copy, PartialEq)]
//pub enum GameMode {
//AI,
//TwoPlayer,
//}
