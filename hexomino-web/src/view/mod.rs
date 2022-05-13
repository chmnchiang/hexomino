use std::rc::Rc;

use api::{GameId, RoomId, WsResult};
use log::{debug, error};
use wasm_bindgen_futures::spawn_local;
use yew::{
    function_component, html, html::Scope, Callback, Component, Context, ContextProvider, Html,
    Properties,
};

use crate::{
    context::{
        connection::{ws::WsListenerToken, ConnectionError, ConnectionStatus},
        MainContext, ScopeExt,
    },
    game::GameMode,
};

use self::{
    game::GameView, login_view::LoginView, room::RoomView, rooms::RoomsView,
    ws_reconnect::WsReconnectModal,
};

mod game;
mod login_view;
mod menu;
mod room;
mod rooms;
mod shared_link;
mod util;
mod ws_reconnect;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <ContextProvider<MainContext> context={MainContext::default()}>
            <MainView/>
        </ContextProvider<MainContext>>
    }
}

#[derive(PartialEq, Properties, Default)]
pub struct MainProps;

pub struct MainView {
    context: MainContext,
    show_reconnect: bool,
    route: Route,
    _ws_listener_token: WsListenerToken,
}

#[derive(PartialEq, Eq)]
pub enum ReconnectStatus {
    Asking,
    Reconnecting,
    None,
}

#[derive(Clone, PartialEq)]
pub enum Route {
    Login,
    Rooms,
    Room { room_id: RoomId },
    Game { game_id: GameId },
}

pub enum MainMsg {
    OnLoginOk,
    OnWsConnected,
    OnConnectionError(ConnectionError),
    OnWsRecv(Rc<WsResult>),
    OnRouteChange(Route),
    Logout,
    ReconnectWs,
}

pub enum Page {
    Menu,
    Game,
}

impl Component for MainView {
    type Message = MainMsg;
    type Properties = MainProps;

    fn create(ctx: &Context<Self>) -> Self {
        let connection_error_callback = ctx.link().callback(MainMsg::OnConnectionError);
        let main_callback = ctx.link().callback(|m| m);
        let (context, _) = ctx.link().context::<MainContext>(Callback::noop()).unwrap();
        context.init_with(connection_error_callback, main_callback);
        let connection = context.connection();
        let status = connection.status();
        Self {
            context,
            route: Route::Login,
            show_reconnect: status == ConnectionStatus::WsNotConnected,
            _ws_listener_token: connection
                .register_ws_callback(ctx.link().callback(MainMsg::OnWsRecv)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use MainMsg::*;
        match msg {
            OnLoginOk => {
                debug!("login ok");
                self.connect_ws(ctx);
                false
            }
            OnWsConnected => {
                ctx.link().main().go(Route::Rooms);
                true
            }
            OnWsRecv(resp) => {
                debug!("get resp = {:?}", resp);
                self.receive_ws_message(ctx, &*resp);
                true
            }
            OnConnectionError(err) => {
                error!("connection error: {err}");
                match err {
                    ConnectionError::Unauthorized => self.logout(ctx),
                    ConnectionError::WsConnectionClose => {
                        self.disconnect_ws();
                    }
                    _ => (),
                }
                true
            }
            OnRouteChange(route) => {
                self.route = route;
                true
            }
            Logout => {
                self.logout(ctx);
                true
            }
            ReconnectWs => {
                self.connect_ws(ctx);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        //let on_choose = ctx.link().callback(MainMsg::StartGame);
        //let on_choose = ctx.link().callback();
        //let inner = match self.page {
        ////Page::Menu => html! { <MenuView {on_choose}/> },
        //Page::Game => html! { <GameView game_mode={self.game_mode.unwrap()}/> },
        //};
        let modal_logout_cb = ctx.link().callback(|_| MainMsg::Logout);
        let modal_reconnect_cb = ctx.link().callback(|_| MainMsg::ReconnectWs);
        html! {
            <main>
                <> { self.route_view(ctx) } </>
                if self.show_reconnect {
                    <WsReconnectModal logout_cb={modal_logout_cb} reconnect_cb={modal_reconnect_cb}/>
                }
            </main>
        }
    }
}

impl MainView {
    fn route_view(&self, ctx: &Context<Self>) -> Html {
        if let Route::Login = self.route {
            return self.login_view(ctx);
        }
        let status = ctx.link().connection().status();
        if status != ConnectionStatus::Connected {
            ctx.link().main().go(Route::Login);
            return html! {};
        }
        let inner = match self.route {
            Route::Login => unreachable!(),
            Route::Rooms => self.rooms_view(),
            Route::Room { room_id } => self.room_view(room_id),
            Route::Game { game_id } => self.game_view(game_id),
        };

        html! {
            <section class="section">
                <div class="container">
                { inner }
                </div>
            </section>
        }
    }

    fn login_view(&self, ctx: &Context<Self>) -> Html {
        let login_callback = ctx.link().callback_once(|()| MainMsg::OnLoginOk);
        html! {
            <LoginView callback={login_callback}/>
        }
    }

    fn rooms_view(&self) -> Html {
        html! {
            <RoomsView/>
        }
    }

    fn room_view(&self, room_id: RoomId) -> Html {
        html! {
            <RoomView {room_id}/>
        }
    }

    fn game_view(&self, _game_id: GameId) -> Html {
        html! {
            <GameView/>
        }
    }
}

impl MainView {
    fn logout(&mut self, ctx: &Context<Self>) {
        self.show_reconnect = false;
        self.context.connection().logout();
        ctx.link().main().go(Route::Login);
    }

    fn connect_ws(&mut self, ctx: &Context<Self>) {
        let context = self.context.clone();
        let link = ctx.link().clone();
        self.show_reconnect = false;
        spawn_local(async move {
            debug!("connecting");
            let result = context.connection().connect_ws().await;
            match result {
                Ok(()) => link.send_message(MainMsg::OnWsConnected),
                Err(err) if let ConnectionError::Unauthorized = err => {
                    link.send_message(MainMsg::OnConnectionError(err))
                }
                Err(_) => {
                    link.send_message(MainMsg::OnConnectionError(ConnectionError::WsConnectionClose))
                }
            }
        });
    }

    fn disconnect_ws(&mut self) {
        self.context.connection().disconnect_ws();
        self.show_reconnect = true;
    }

    fn receive_ws_message(&self, ctx: &Context<Self>, msg: &WsResult) {
        use api::WsResponse::*;
        match msg {
            MoveToRoom(room_id) => {
                ctx.link().main().go(Route::Room { room_id: *room_id });
            }
            GameStart(game_info) => {
                ctx.link().main().go(Route::Game {
                    game_id: game_info.game_id,
                });
            }
            _ => (),
        }
    }
}
