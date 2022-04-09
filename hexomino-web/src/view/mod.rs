use yew::{html, Component, Context, Html, Properties};

use crate::{
    game::GameMode,
    view::{game::GameView, menu::MenuView},
};

mod game;
mod menu;
mod shared_link;

#[derive(PartialEq, Properties, Default)]
pub struct MainProps;

pub struct MainView {
    page: Page,
    game_mode: Option<GameMode>,
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
            page: Page::Menu,
            game_mode: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        use MainMsg::*;
        match msg {
            StartGame(mode) => {
                self.page = Page::Game;
                self.game_mode = Some(mode);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_choose = ctx.link().callback(MainMsg::StartGame);
        let inner = match self.page {
            Page::Menu => html! { <MenuView {on_choose}/> },
            Page::Game => html! { <GameView game_mode={self.game_mode.unwrap()}/> },
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
