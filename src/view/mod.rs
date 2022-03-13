use yew::{html, Component, Context, Html, Properties};

use crate::view::game::GameComponent;

mod game;

#[derive(PartialEq, Properties, Default)]
pub struct MainProps;

pub struct MainComponent;

impl Component for MainComponent {
    type Message = ();
    type Properties = MainProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <section class="section">
                <p> { "Main view" } </p>
                <GameComponent></GameComponent>
            </section>
        }
    }
}
