use std::rc::Rc;

use anyhow::{Error, Result};
use api::WsResult;
use log::{debug, error};
use yew::{function_component, html, Component, Context, ContextProvider, Html, Properties};
use yew_router::{history::History, prelude::RouterScopeExt, BrowserRouter, Routable, Switch};

use crate::{
    context::{self, Connection, ConnectionError},
    game::GameMode,
    util::{FutureExt as _, ResultExt},
};

use self::{login_view::LoginView, rooms::RoomsView};

mod game;
mod login_view;
mod menu;
mod rooms;
mod shared_link;
mod util;

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
    LoginOk,
    WsConnected,
    ConnectionFailed,
    ConnectionError(ConnectionError),
    ServerResp(WsResult),
}

pub enum Page {
    Menu,
    Game,
}

impl Component for MainView {
    type Message = MainMsg;
    type Properties = MainProps;

    fn create(ctx: &Context<Self>) -> Self {
        let connection_error_callback = ctx.link().callback(MainMsg::ConnectionError);
        Self {
            context: Rc::new(MainContextInner {
                connection: Connection::new(connection_error_callback),
            }),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use MainMsg::*;
        match msg {
            LoginOk => {
                debug!("login ok");
                let fut = self
                    .context
                    .connection
                    .connect_ws(ctx.link().callback(ServerResp));
                let callback_ok = ctx.link().callback(|()| WsConnected);
                let callback_err = ctx.link().callback(|_| ConnectionFailed);
                let handler = move |res: context::Result<()>| {
                    let _ = res.log_err().map_err(move |err| callback_err.emit(err));
                    callback_ok.emit(());
                };
                fut.spawn_with_handler(handler);
                false
            }
            WsConnected => {
                ctx.link().history().unwrap().push(Route::Rooms);
                true
            }
            ServerResp(resp) => {
                debug!("get resp = {:?}", resp);
                false
            }
            ConnectionError(err) => {
                error!("connection error: {err}");
                match err {
                    context::ConnectionError::Unauthorized => self.context.connection.logout(),
                    context::ConnectionError::WsConnectionClose => {
                        self.context.connection.disconnect_ws()
                    }
                    _ => (),
                }
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
        html! {
            <ContextProvider<MainContext> context={self.context.clone()}>
                <main>
                    <Switch<Route> render={Switch::render(self.switch(ctx))}/>
                </main>
            </ContextProvider<MainContext>>
        }
    }
}

impl MainView {
    fn switch(&self, ctx: &Context<Self>) -> impl Fn(&Route) -> Html + 'static {
        let login_callback = ctx.link().callback_once(|()| MainMsg::LoginOk);

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
}
