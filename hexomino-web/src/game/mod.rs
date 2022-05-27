use std::{cell::RefCell, rc::Rc};

use anyhow::{Context, Result};
use api::MatchInfo;
use getset::{CopyGetters, Getters};
use hexomino_core::{Action, Player};
use yew::Callback;

use crate::util::Shared;

#[derive(Getters)]
pub struct MatchState {
    info: MatchInfo,
    game_idx: i32,
    scores: [u32; 2],
    #[getset(get = "pub")]
    game: Option<SharedGameState>,
}

pub type SharedGameState = Shared<GameState>;
#[derive(Getters, CopyGetters)]
pub struct GameState {
    #[getset(get = "pub")]
    core: hexomino_core::State,
    #[getset(get_copy = "pub")]
    me: Player,
    #[getset(get = "pub")]
    player_names: [String; 2],
    num_action: usize,
}

impl MatchState {
    pub fn from_api(match_state: api::MatchState) -> Self {
        let (game, game_idx) = if match_state.game_idx == -1 {
            (None, -1)
        } else {
            let mut game = GameState::new(
                match_state.you,
                player_names(&match_state.info, match_state.you),
            );
            for action in match_state.prev_actions {
                let _ = game.core.play(action);
            }
            (Some(Rc::new(RefCell::new(game))), match_state.game_idx)
        };

        MatchState {
            info: match_state.info,
            game_idx,
            scores: match_state.scores,
            game,
        }
    }

    pub fn player_name(&self, idx: usize) -> &str {
        &self.info.user_data[idx].name
    }

    pub fn scores(&self, idx: usize) -> u32 {
        self.scores[idx]
    }

    pub fn new_game(&mut self, me: Player) {
        self.game = Some(Rc::new(RefCell::new(GameState::new(
            me,
            player_names(&self.info, me)
        ))));
    }

    pub fn update(&mut self, action: Action) -> Result<()> {
        let game = self.game.as_ref().context("game has not been initialized")?;
        game.borrow_mut().core.play(action)
    }
}

fn player_names(info: &MatchInfo, me: Player) -> [String; 2] {
    let mut player_names = [0, 1].map(|idx| info.user_data[idx].name.clone());
    if me == Player::Second {
        player_names.swap(0, 1);
    }
    player_names
}


impl GameState {
    fn new(me: Player, player_names: [String; 2]) -> Self {
        Self {
            core: hexomino_core::State::new(),
            me,
            player_names,
            num_action: 0,
        }
    }

    pub fn name_of(&self, player: Player) -> &str {
        &self.player_names[player.id()]
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
