use std::rc::Rc;

use api::{UserStatus, WsResult};
use hexomino_core::Hexo;
use wasm_bindgen_futures::spawn_local;
use yew::{
    classes, function_component, html, Callback, Component, Context, ContextProvider, Html,
    Properties,
};

use crate::context::{
    connection::{ws::WsListenerToken, ConnectionError, ConnectionStatus},
    MainContext, ScopeExt,
};

use self::{
    common::hexo_svg::HexoSvg,
    game::{ai_game_view::AiGameView, GameView},
    login_view::LoginView,
    match_history_view::MatchHistoryView,
    room::RoomView,
    rooms::RoomsView,
    ws_reconnect::WsReconnectModal, error_message::ErrorMessageView,
};

mod common;
mod game;
mod login_view;
mod match_history_view;
mod room;
mod rooms;
mod shared_link;
mod util;
mod ws_reconnect;
mod error_message;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <ContextProvider<MainContext> context={MainContext::default()}>
            <MainView/>
        </ContextProvider<MainContext>>
    }
}

#[derive(PartialEq, Eq, Properties, Default)]
pub struct MainProps;

pub struct MainView {
    context: MainContext,
    show_reconnect: bool,
    show_mobile_navbar: bool,
    route: Route,
    _ws_listener_token: WsListenerToken,
}

#[derive(PartialEq, Eq)]
pub enum ReconnectStatus {
    Asking,
    Reconnecting,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Route {
    Login,
    Rooms,
    Room,
    Game,
    AiGame,
    MatchHistory,
}

pub enum MainMsg {
    OnLoginOk,
    OnWsConnected,
    OnConnectionError(ConnectionError),
    OnWsRecv(Rc<WsResult>),
    OnRouteChange(Route),
    Logout,
    ReconnectWs,
    ToggleMobileNav,
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
        let (context, _) = ctx
            .link()
            .context::<MainContext>(Callback::noop())
            .expect("cannot get main context");
        context.init_with(connection_error_callback, ctx.link().callback(|m| m));
        let connection = context.connection();
        let status = connection.status();
        Self {
            context,
            route: Route::Login,
            show_reconnect: status == ConnectionStatus::WsNotConnected,
            show_mobile_navbar: false,
            _ws_listener_token: connection
                .register_ws_callback(ctx.link().callback(MainMsg::OnWsRecv)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use MainMsg::*;
        match msg {
            OnLoginOk => {
                log::debug!("Login completed.");
                self.connect_ws(ctx);
                false
            }
            OnWsConnected => {
                log::info!("Websocket connected.");
                ctx.link().main().go(Route::Rooms);
                true
            }
            OnWsRecv(resp) => {
                log::debug!("Get websocket message: {:?}", resp);
                self.receive_ws_message(ctx, &*resp);
                true
            }
            OnConnectionError(err) => {
                log::error!("Connection error: {err}");
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
                log::debug!("Route changed to {:?}", route);
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
            ToggleMobileNav => {
                self.show_mobile_navbar = !self.show_mobile_navbar;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let modal_logout_cb = ctx.link().callback(|_| MainMsg::Logout);
        let modal_reconnect_cb = ctx.link().callback(|_| MainMsg::ReconnectWs);
        html! {
            <main>
                { self.route_view(ctx) }
                if self.show_reconnect {
                    <WsReconnectModal logout_cb={modal_logout_cb} reconnect_cb={modal_reconnect_cb}/>
                }
                <ErrorMessageView/>
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
            Route::Room => self.room_view(),
            Route::Game => self.game_view(),
            Route::AiGame => self.ai_game_view(),
            Route::MatchHistory => self.match_history_view(),
        };
        let has_navbar = matches!(
            self.route,
            Route::Rooms | Route::AiGame | Route::MatchHistory
        );

        html! {
            <>
                {
                    if has_navbar {
                        self.navbar_view(self.route, ctx)
                    } else {
                        html!()
                    }
                }
                <section class="section">
                    <div class="container">
                    { inner }
                    </div>
                </section>
            </>
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

    fn room_view(&self) -> Html {
        html! {
            <RoomView/>
        }
    }

    fn game_view(&self) -> Html {
        html! {
            <GameView/>
        }
    }

    fn ai_game_view(&self) -> Html {
        html! {
            <AiGameView/>
        }
    }

    fn match_history_view(&self) -> Html {
        html! {
            <MatchHistoryView/>
        }
    }

    fn navbar_view(&self, route: Route, ctx: &Context<Self>) -> Html {
        let main = ctx.link().main();
        let navbar_item_html = move |target: Route, text: &str| -> Html {
            let main = main.clone();
            let onclick = move |_| main.go(target);
            html! {
                <a class="navbar-item" href="javascript:void(0)" {onclick}>
                    <span class={classes!("navbar-route", (route == target).then_some("is-active"))}>{ text }</span>
                </a>
            }
        };
        let my_name = ctx
            .link()
            .connection()
            .me()
            .map(|user| user.name.clone())
            .unwrap_or_else(|| "<Unknown>".to_string());
        let logout_onclick = ctx.link().callback(|_| MainMsg::Logout);
        let mobile_burger_onclick = ctx.link().callback(|_| MainMsg::ToggleMobileNav);

        html! {
            <nav class="navbar is-light" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <div class={classes!["navbar-item", self.show_mobile_navbar.then_some("is-active")]}>
                        <div style="width: 30px; height: 30px;
                            transform: rotate(180deg) scaleX(-1) scale(1.5)">
                            <HexoSvg hexo={Hexo::new(6)}/>
                        </div>
                        <b> { "Hexomino" } </b>
                    </div>
                    <a role="button" class="navbar-burger" aria-label="menu" aria-expanded="false"
                        onclick={mobile_burger_onclick}>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                </div>
                <div class={classes!["navbar-menu", self.show_mobile_navbar.then_some("is-active")]}>
                    <div class="navbar-start">
                        {
                            [(Route::Rooms, "Public Games"),
                             (Route::AiGame, "AI Game"),
                             (Route::MatchHistory, "Match History")].iter()
                                 .map(|(target, text)| navbar_item_html(*target, text))
                                 .collect::<Html>()
                        }
                    </div>
                    <div class="navbar-end">
                        <div class="navbar-item">{ format!("Welcome, {}", my_name) }</div>
                        <a class="navbar-item" href="javascript:void(0)" onclick={logout_onclick}>
                            <span class="icon"><i class="fa-solid fa-arrow-right-from-bracket"></i></span>
                            <span>{ "Logout" }</span>
                        </a>
                        <a class="navbar-item" href="https://github.com/chmnchiang/hexomino/issues" target="_blank">
                            <span class="icon"><i class="fa-solid fa-bug"></i></span>
                            <span>{ "Report bug" }</span>
                        </a>
                    </div>
                </div>
            </nav>
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
        if let UserStatusUpdate(status) = msg {
            let next_route = match status {
                UserStatus::Idle => Some(Route::Rooms),
                UserStatus::InRoom => Some(Route::Room),
                UserStatus::InGame => Some(Route::Game),
            };
            if let Some(next_route) = next_route {
                ctx.link().main().go(next_route);
            }
        }
    }
}
