use std::rc::Rc;

use api::WsResult;
use log::{debug, error};
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, Component, Context, ContextProvider, Html, Properties};
use yew_router::{history::History, prelude::RouterScopeExt, BrowserRouter, Routable, Switch};

use crate::{
    context::{self, Connection, ConnectionError, ConnectionStatus},
    game::GameMode,
    util::{FutureExt as _, ResultExt},
};

use self::{login_view::LoginView, rooms::RoomsView, ws_reconnect::WsReconnectModal};

mod game;
mod login_view;
mod menu;
mod rooms;
mod shared_link;
mod util;
mod ws_reconnect;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <MainView/>
        </BrowserRouter>
    }
}

#[derive(PartialEq, Properties, Default)]
pub struct MainProps;

pub struct MainView {
    context: MainContext,
    show_reconnect: bool,
}

#[derive(PartialEq, Eq)]
pub enum ReconnectStatus {
    Asking,
    Reconnecting,
    None,
}

type MainContext = Rc<MainContextInner>;

#[derive(PartialEq, Eq)]
struct MainContextInner {
    connection: Connection,
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Login,
    #[at("/rooms")]
    Rooms,
}

pub enum MainMsg {
    OnLoginOk,
    OnWsConnected,
    OnConnectionError(ConnectionError),
    OnServerResp(WsResult),
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
        Self {
            context: Rc::new(MainContextInner {
                connection: Connection::new(connection_error_callback),
            }),
            show_reconnect: false,
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
                ctx.link().history().unwrap().push(Route::Rooms);
                true
            }
            OnServerResp(resp) => {
                debug!("get resp = {:?}", resp);
                false
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
            Logout => {
                self.logout(ctx);
                true
            }
            ReconnectWs => {
                self.connect_ws(ctx);
                true
            }
            _ => false,
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
            <ContextProvider<MainContext> context={self.context.clone()}>
                <main>
                    <Switch<Route> render={Switch::render(self.switch(ctx))}/>
                </main>
                if self.show_reconnect {
                    <WsReconnectModal logout_cb={modal_logout_cb} reconnect_cb={modal_reconnect_cb}/>
                }
            </ContextProvider<MainContext>>
        }
    }
}

impl MainView {
    fn switch(&self, ctx: &Context<Self>) -> impl Fn(&Route) -> Html + 'static {
        let login_callback = ctx.link().callback_once(|()| MainMsg::OnLoginOk);

        //let rooms_link = SharedLink::new();
        //ctx.link()
        //.send_message(MainMsg::SetRoomLink(rooms_link.downgrade()));

        move |route| {
            use Route::*;
            match route {
                Login => html! {
                    <LoginView callback={login_callback.clone()}/>
                },
                Rooms => html! {
                    <RoomsView/>
                },
            }
        }
    }

    fn logout(&mut self, ctx: &Context<Self>) {
        self.show_reconnect = false;
        self.context.connection.logout();
        ctx.link().history().unwrap().push(Route::Login);
    }

    fn connect_ws(&mut self, ctx: &Context<Self>) {
        let context = self.context.clone();
        let link = ctx.link().clone();
        self.show_reconnect = false;
        spawn_local(async move {
            debug!("connecting");
            let result = context
                .connection
                .connect_ws(link.callback(MainMsg::OnServerResp))
                .await;
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
        self.context.connection.disconnect_ws();
        self.show_reconnect = true;
    }
}
