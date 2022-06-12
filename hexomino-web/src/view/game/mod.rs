use std::rc::Rc;

use api::{
    GameEndInfo, MatchAction, MatchActionApi, MatchEndInfo, MatchEvent, SyncMatchApi, UserPlay,
    WsResponse, WsResult,
};
use hexomino_core::{Action, GamePhase, Player};
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html, Properties};

use self::{end_view::EndView, pick_view::PickView, place_view::PlaceView};
use crate::{
    context::{connection::ws::WsListenerToken, ScopeExt},
    game::{MatchError, MatchInnerState, MatchPhase, MatchState, SharedGameState},
    util::ResultExt,
    view::game::{match_end_view::MatchEndView, turn_indicator::TurnIndicator},
};

pub mod ai_game_view;
mod board_canvas;
mod board_renderer;
mod bottom_message;
mod end_view;
mod hexo_block;
mod hexo_table;
mod match_end_view;
mod pick_view;
mod place_view;
mod turn_indicator;

#[derive(PartialEq, Eq, Properties)]
pub struct GameProps {}

pub struct GameView {
    mtch: Option<MatchState>,
    game_status: GameStatus,
    _ws_listener_token: WsListenerToken,
}

pub enum GameMsg {
    OnSyncMatch(api::MatchState),
    OnStartGame(Player),
    OnUserPlay(UserPlay),
    OnGameEnd(GameEndInfo),
    OnMatchEnd(MatchEndInfo),
    UserPlay(Action),
}

#[derive(PartialEq, Eq)]
pub enum GameStatus {
    NotStarted,
    Playing,
    Ended,
}

impl Component for GameView {
    type Message = GameMsg;
    type Properties = GameProps;

    fn create(ctx: &Context<Self>) -> Self {
        let connection = ctx.link().connection();
        let ws_listener_token =
            connection.register_ws_callback(ctx.link().batch_callback(|resp: Rc<WsResult>| {
                match (&*resp).clone() {
                    WsResponse::MatchEvent(MatchEvent::GameStart { you }) => {
                        Some(GameMsg::OnStartGame(you))
                    }
                    WsResponse::MatchEvent(MatchEvent::UserPlay(action)) => {
                        Some(GameMsg::OnUserPlay(action))
                    }
                    WsResponse::MatchEvent(MatchEvent::GameEnd(info)) => {
                        Some(GameMsg::OnGameEnd(info))
                    }
                    WsResponse::MatchEvent(MatchEvent::MatchEnd(info)) => {
                        Some(GameMsg::OnMatchEnd(info))
                    }
                    _ => None,
                }
            }));
        Self {
            mtch: None,
            game_status: GameStatus::NotStarted,
            _ws_listener_token: ws_listener_token,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: GameMsg) -> bool {
        use GameMsg::*;
        match msg {
            OnSyncMatch(match_state) => self.on_sync_match(match_state),
            OnStartGame(info) => self.on_start_game(info, ctx),
            OnGameEnd(info) => self.on_game_end(info, ctx),
            OnUserPlay(action) => self.on_user_play(action, ctx),
            OnMatchEnd(info) => self.on_match_end(info, ctx),
            UserPlay(action) => self.user_play(action, ctx),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        fn loader_html() -> Html {
            html! {
                <div class="pageloader is-active">
                    <span class="title">{"Waiting for the game to start..."}</span>
                </div>
            }
        }
        let Some(mtch) = &self.mtch else { return loader_html(); };
        match mtch.state() {
            MatchInnerState::NotStarted => loader_html(),
            MatchInnerState::Ended { winner } => {
                let info = MatchEndInfo {
                    scores: *mtch.scores(),
                    winner: *winner,
                };
                html! {
                    <MatchEndView {info} names={mtch.names()}/>
                }
            }
            MatchInnerState::Playing(game_state) => self.game_playing_view(mtch, game_state, ctx),
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            do_sync_match(ctx);
        }
    }
}

impl GameView {
    fn on_sync_match(&mut self, mtch: api::MatchState) -> bool {
        self.mtch = Some(MatchState::from_api(mtch));
        true
    }

    fn on_start_game(&mut self, me: Player, ctx: &Context<Self>) -> bool {
        let Some(mtch) = self.match_mut_or_sync(ctx) else { return false };
        is_ok_or_sync(mtch.update_new_game(me), ctx);
        true
    }

    fn on_user_play(&mut self, action: UserPlay, ctx: &Context<Self>) -> bool {
        let Some(mtch) = self.match_mut_or_sync(ctx) else { return false };
        is_ok_or_sync(mtch.update_action(action), ctx);
        true
    }

    fn on_game_end(&mut self, info: GameEndInfo, ctx: &Context<Self>) -> bool {
        let Some(mtch) = self.match_mut_or_sync(ctx) else { return false };
        is_ok_or_sync(mtch.update_game_end(info), ctx);
        true
    }

    fn user_play(&mut self, action: Action, ctx: &Context<Self>) -> bool {
        let Some(mtch) = self.match_mut_or_sync(ctx) else { return false };
        let MatchInnerState::Playing(game) = mtch.state() else { return false };
        let game = game.borrow();
        if !game
            .core()
            .current_player()
            .is_some_and(|&player| player == game.me())
        {
            return false;
        }
        let connection = ctx.link().connection();
        spawn_local(async move {
            let _ = connection
                .post_api::<MatchActionApi>("/api/game/action", MatchAction::Play(action))
                .await
                .log_err();
        });
        false
    }

    fn on_match_end(&mut self, info: MatchEndInfo, ctx: &Context<Self>) -> bool {
        let Some(mtch) = self.match_mut_or_sync(ctx) else { return false };
        is_ok_or_sync(mtch.update_match_end(info), ctx);
        true
    }

    fn match_mut_or_sync(&mut self, ctx: &Context<Self>) -> Option<&mut MatchState> {
        if let Some(mtch) = &mut self.mtch {
            Some(mtch)
        } else {
            do_sync_match(ctx);
            None
        }
    }

    fn game_playing_view(
        &self,
        mtch: &MatchState,
        game_state: &SharedGameState,
        ctx: &Context<Self>,
    ) -> Html {
        let send_pick = ctx
            .link()
            .callback(|hexo| GameMsg::UserPlay(Action::Pick(hexo)));
        let send_place = ctx
            .link()
            .callback(|moved_hexo| GameMsg::UserPlay(Action::Place(moved_hexo)));
        let game_state_borrow = game_state.borrow();
        let me = game_state_borrow.me();
        let core_game_state = game_state_borrow.core();

        html! {
            <div>
                <TurnIndicator {me}
                    current_player={core_game_state.current_player()}
                    player_names={mtch.names_ord_by_player()}
                    scores={mtch.scores_ord_by_player()}/>
                {
                    match mtch.phase() {
                        MatchPhase::GamePlaying => {
                            match core_game_state.phase() {
                                GamePhase::Pick => html!{
                                    <PickView state={game_state.clone()} send_pick={send_pick}/>
                                },
                                GamePhase::Place | GamePhase::End => html!{
                                    <PlaceView state={game_state.clone()} send_place={send_place}/>
                                },
                            }
                        }
                        MatchPhase::GameEnded => html!{
                            <EndView state={game_state.clone()} names={mtch.names_ord_by_player()}/>
                        },
                        _ => html!(),
                    }
                }
            </div>
        }
    }
}

fn do_sync_match(ctx: &Context<GameView>) {
    let callback = ctx.link().callback(GameMsg::OnSyncMatch);
    let connection = ctx.link().connection();
    spawn_local(async move {
        let resp = connection
            .post_api::<SyncMatchApi>("/api/game/sync", ())
            .await;
        let Ok(result) = resp.log_err() else { return };
        let Ok(match_state) = result.log_err() else { return };
        callback.emit(match_state);
    })
}

fn is_ok_or_sync(result: Result<(), MatchError>, ctx: &Context<GameView>) -> bool {
    match result {
        Ok(_) => true,
        Err(MatchError::StateNotSynced) => {
            do_sync_match(ctx);
            false
        }
        _ => false,
    }
}
