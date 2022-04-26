use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

use crate::context::{self, ConnectionError};

use super::MainContext;

#[derive(Default)]
pub struct LoginView {
    username: NodeRef,
    password: NodeRef,
    error: Option<String>,
}

pub enum LoginMsg {
    Login,
    LoginFailed(ConnectionError),
}

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub callback: Callback<()>,
}

impl Component for LoginView {
    type Message = LoginMsg;
    type Properties = LoginProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use LoginMsg::*;
        match msg {
            Login => {
                let username = self.username.cast::<HtmlInputElement>().unwrap().value();
                let password = self.password.cast::<HtmlInputElement>().unwrap().value();

                let (context, _): (MainContext, _) = ctx
                    .link()
                    .context(Callback::noop())
                    .expect("get context failed");

                let callback_ok = ctx.props().callback.clone();
                let callback_err = ctx.link().callback(LoginFailed);
                let fut = async move {
                    match context.connection.login(username, password).await {
                        Ok(_) => callback_ok.emit(()),
                        Err(err) => callback_err.emit(err),
                    }
                };
                spawn_local(fut);

                false
            }
            LoginMsg::LoginFailed(err) => {
                self.error = Some(err.to_string());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| LoginMsg::Login);

        html! {
            <session class="hero is-primary is-fullheight">
                <div class="hero-body">
                    <div class="container">
                        <div class="columns is-centered">
                            <div class="column is-4 is-6-tablet">
                                <div class="box">
                                    <div class="field">
                                        <label class="label">{"Username"}</label>
                                        <div class="control">
                                            <input ref={self.username.clone()} class="input" type="text"/>
                                        </div>
                                    </div>
                                    <div class="field">
                                        <label class="label">{"Password"}</label>
                                        <div class="control">
                                            <input ref={self.password.clone()} class="input" type="text"/>
                                        </div>
                                    </div>
                                    <div class="field is-grouped is-grouped-right">
                                        <div class="control">
                                            <button class="button is-success" onclick={submit}>{"Login"}</button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </session>
        }
    }
}
