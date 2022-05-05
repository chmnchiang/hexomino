use std::rc::Rc;

use api::{WsResult, RoomId};
use log::{debug, error};
use wasm_bindgen_futures::spawn_local;
use yew::{
    function_component, html, html::Scope, Callback, Component, Context, ContextProvider, Html,
    Properties,
};
use yew_router::{
    components::Redirect, history::History, prelude::RouterScopeExt, BrowserRouter, Routable,
    Switch,
};

use crate::{
    context::{
        connection::{ws::WsListenerToken, ConnectionError, ConnectionStatus},
        MainContext, ScopeExt,
    },
    game::GameMode,
};

use self::{login_view::LoginView, rooms::RoomsView, ws_reconnect::WsReconnectModal, room::RoomView};

mod game;
mod login_view;
mod menu;
mod rooms;
mod room;
mod shared_link;
mod util;
mod ws_reconnect;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <ContextProvider<MainContext> context={MainContext::default()}>
                <MainView/>
            </ContextProvider<MainContext>>
        </BrowserRouter>
    }
}

#[derive(PartialEq, Properties, Default)]
pub struct MainProps;

pub struct MainView {
    context: MainContext,
    show_reconnect: bool,
    _ws_listener_token: WsListenerToken,
}

#[derive(PartialEq, Eq)]
pub enum ReconnectStatus {
    Asking,
    Reconnecting,
    None,
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Login,
    #[at("/rooms")]
    Rooms,
    #[at("/room/:room_id")]
    Room { room_id: RoomId },
}

pub enum MainMsg {
    OnLoginOk,
    OnWsConnected,
    OnConnectionError(ConnectionError),
    OnWsRecv(Rc<WsResult>),
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
        let (context, _) = ctx.link().context::<MainContext>(Callback::noop()).unwrap();
        context.init_with(connection_error_callback);
        let connection = context.connection();
        let status = connection.status();
        Self {
            context,
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
                ctx.link().history().unwrap().push(Route::Rooms);
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
                <Switch<Route> render={Switch::render(switch(ctx.link()))}/>
                if self.show_reconnect {
                    <WsReconnectModal logout_cb={modal_logout_cb} reconnect_cb={modal_reconnect_cb}/>
                }
            </main>
        }
    }
}

type MainLink = Scope<MainView>;

fn switch(link: &MainLink) -> impl Fn(&Route) -> Html + 'static {
    let link = link.clone();
    move |route| {
        use Route::*;
        match route {
            Login => switch_login(&link),
            Rooms => ensure_login(&link, switch_rooms),
            Room { room_id } => ensure_login(&link, switch_room(*room_id)),
        }
    }
}

fn switch_login(link: &MainLink) -> Html {
    //if link.connection().status() == ConnectionStatus::Connected {
    //return html! {
    //<Redirect<Route> to={Route::Rooms}/>
    //}
    //}
    let login_callback = link.callback_once(|()| MainMsg::OnLoginOk);
    html! {
        <LoginView callback={login_callback}/>
    }
}

fn ensure_login(link: &MainLink, then: impl Fn(&MainLink) -> Html) -> Html {
    let status = link.connection().status();
    if status != ConnectionStatus::Connected {
        return html! {
            <Redirect<Route> to={Route::Login}/>
        };
    }
    then(link)
}

fn with_default_wrapper(html: Html) -> Html {
    html! {
        <section class="section">
            <div class="container">
                { html }
            </div>
        </section>
    }
}

fn switch_rooms(_link: &MainLink) -> Html {
    with_default_wrapper(html! {
        <RoomsView/>
    })
}

fn switch_room(room_id: RoomId) -> impl Fn(&MainLink) -> Html {
    move |_| with_default_wrapper(html! {
        <RoomView {room_id}/>
    })
}


impl MainView {
    fn logout(&mut self, ctx: &Context<Self>) {
        self.show_reconnect = false;
        self.context.connection().logout();
        ctx.link().history().unwrap().push(Route::Login);
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
                ctx.link().history().unwrap().push(Route::Room { room_id: *room_id });
            }
            _ => (),
        }
    }
}
