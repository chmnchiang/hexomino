use yew::{html, Callback, Component, Context, Properties};

use super::GameMode;

#[derive(PartialEq, Properties)]
pub struct MenuProps {
    pub on_choose: Callback<GameMode>,
}

pub struct MenuView;

impl Component for MenuView {
    type Message = ();
    type Properties = MenuProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let onclick_single_player = ctx.props().on_choose.reform(|_| GameMode::AI);
        let onclick_two_player = ctx.props().on_choose.reform(|_| GameMode::TwoPlayer);
        html! {
            <>
                <div class="columns is-centered">
                    <div class="column is-half">
                        <button class="button is-primary" style="width: 100%; height: 5rem;"
                            onclick={onclick_single_player}>
                            <p style="font-size: 2rem;"> {"Single Player (vs AI)"} </p>
                        </button>
                    </div>
                </div>
                <div class="columns">
                    <div class="column is-half is-offset-one-quarter">
                        <button class="button is-primary" style="width: 100%; height: 5rem;"
                            onclick={onclick_two_player}>
                            <p style="font-size: 2rem;"> {"Two Player"} </p>
                        </button>
                    </div>
                </div>
            </>
        }
    }
}
