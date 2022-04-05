use yew::{html, Component, Context, Html, Properties};

use crate::view::{game::GameView, menu::MenuView};

mod game;
mod menu;
mod util;

#[derive(PartialEq, Properties, Default)]
pub struct MainProps;

pub struct MainView {
    current_page: Page,
}

pub enum GameMode {
    AI,
    TwoPlayer,
}

pub enum MainMsg {
    StartGame(GameMode),
}

pub enum Page {
    Menu,
    Game,
}

impl Component for MainView {
    type Message = MainMsg;
    type Properties = MainProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            //current_page: Page::Menu,
            current_page: Page::Game,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        use MainMsg::*;
        match msg {
            StartGame(_mode) => self.current_page = Page::Game,
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_choose = ctx.link().callback(MainMsg::StartGame);
        let inner = match self.current_page {
            Page::Menu => html! { <MenuView {on_choose}/> },
            Page::Game => html! { <GameView/> },
        };
        html! {
            <main>
                <section class="section">
                    <div class="container is-widescreen">
                        { inner }
                    </div>
                </section>
            </main>
        }
    }
}
