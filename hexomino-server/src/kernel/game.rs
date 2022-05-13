use api::{GameAction, GameError, GameEvent, GameId, UserId, WsResponse};
use getset::CopyGetters;
use hexomino_core::{Action, Player, State};
use uuid::Uuid;

use crate::result::ApiResult;

use super::{
    actor::{Actor, Addr, Context, Handler},
    user::User,
};

type Result<T> = ApiResult<T, GameError>;

#[derive(Clone, derivative::Derivative)]
#[derivative(Debug)]
#[derive(CopyGetters)]
pub struct GameHandle {
    #[getset(get_copy = "pub")]
    id: GameId,
    #[derivative(Debug = "ignore")]
    addr: Addr<GameActor>,
}

impl GameHandle {
    pub async fn send_user_action(&self, user_id: UserId, action: GameAction) -> Result<()> {
        self.addr.send(UserAction { user_id, action }).await
    }
}

// TODO: derive Debug for state
pub struct GameActor {
    id: GameId,
    state: GameState,
    users: [User; 2],
}

impl GameActor {
    pub fn new(users: [User; 2]) -> Self {
        Self {
            id: GameId(Uuid::new_v4()),
            users,
            state: GameState::new(),
        }
    }

    pub fn start(self) -> GameHandle {
        let id = self.id;
        let addr = Actor::start(self);
        GameHandle { id, addr }
    }

    fn get_player(&self, user_id: UserId) -> Option<Player> {
        if self.users[0].id() == user_id {
            Some(Player::First)
        } else if self.users[1].id() == user_id {
            Some(Player::Second)
        } else {
            None
        }
    }

    fn broadcast_action(&self, action: Action) {
        tracing::debug!("broadcast action");
        for users in &self.users {
            users.do_send(WsResponse::GameEvent(GameEvent::UserPlay(action)));
        }
    }
}

impl Actor for GameActor {}

pub struct UserAction {
    pub user_id: UserId,
    pub action: GameAction,
}

impl Handler<UserAction> for GameActor {
    type Output = Result<()>;

    fn handle(&mut self, msg: UserAction, _ctx: &Context<Self>) -> Self::Output {
        let player = self.get_player(msg.user_id).ok_or(GameError::NotInGame)?;
        let pid = player.id();
        match msg.action {
            GameAction::Connected => {
                self.state.players[pid].is_connected = true;
            }
            GameAction::Play(action) => {
                if self.state.game.current_player() != Some(player) {
                    return Err(GameError::NotYourTurn.into());
                }
                self.state
                    .game
                    .play(action)
                    .map_err(|err| GameError::GameActionError(format!("{err}")))?;
                self.broadcast_action(action);
            }
        }
        Ok(())
    }
}

struct GameState {
    game: State,
    players: [PlayerState; 2],
}

struct PlayerState {
    is_connected: bool,
}

impl GameState {
    fn new() -> Self {
        Self {
            game: State::new(),
            players: [PlayerState::new(), PlayerState::new()],
        }
    }
}

impl PlayerState {
    fn new() -> Self {
        Self {
            is_connected: false,
        }
    }
}
