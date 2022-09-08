use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::{html, Callback, Component, Context, Html, NodeRef, Properties, classes};

use crate::context::{ScopeExt, connection::{ConnectionError, ConnectionStatus}};



#[derive(Default)]
pub struct LoginView {
    username: NodeRef,
    password: NodeRef,
    error: Option<String>,
    is_loading: bool,
}

pub enum LoginMsg {
    Login,
    LoginFailed(ConnectionError),
    LoadComplete(bool),
}

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub callback: Callback<()>,
}

impl Component for LoginView {
    type Message = LoginMsg;
    type Properties = LoginProps;

    fn create(ctx: &Context<Self>) -> Self {
        let load_auth_callback = ctx.link().callback(LoginMsg::LoadComplete);
        let connection = ctx.link().connection();
        if connection.status() == ConnectionStatus::LoggedOut {
            spawn_local(async move {
                let result = connection.load_auth().await;
                load_auth_callback.emit(result.is_ok());
            });
        }
        Self {
            is_loading: true,
            ..Self::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use LoginMsg::*;
        match msg {
            Login => {
                let username = self.username.cast::<HtmlInputElement>().unwrap().value();
                let password = if cfg!(feature = "competition-mode") {
                    self.password.cast::<HtmlInputElement>().unwrap().value()
                } else {
                    "".to_string()
                };

                let connection = ctx.link().connection();
                let callback_ok = ctx.props().callback.clone();
                let callback_err = ctx.link().callback(LoginFailed);
                let fut = async move {
                    match connection.login(username, password).await {
                        Ok(_) => callback_ok.emit(()),
                        Err(err) => callback_err.emit(err),
                    }
                };
                spawn_local(fut);
                self.is_loading = true;
            }
            LoginMsg::LoginFailed(err) => {
                self.error = Some(err.to_string());
                self.is_loading = false;
            }
            LoadComplete(auth_ok) => {
                if auth_ok {
                    ctx.props().callback.emit(());
                }
                self.is_loading = false;
            }
        }
        true
    }

    #[cfg(feature = "competition-mode")]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| LoginMsg::Login);
        let is_loading = self.is_loading.then(|| "is-loading");

        html! {
            <section class="hero is-primary is-fullheight">
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
                                            <button class={classes!("button", "is-success", is_loading)}
                                                onclick={submit}>{"Login"}</button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </section>
        }
    }

    #[cfg(not(feature = "competition-mode"))]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| LoginMsg::Login);
        let is_loading = self.is_loading.then(|| "is-loading");

        html! {
            <section class="hero is-primary is-fullheight">
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
                                    <div class="field is-grouped is-grouped-right">
                                        <div class="control">
                                            <button class={classes!("button", "is-success", is_loading)}
                                                onclick={submit}>{"Login"}</button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </section>
        }
    }
}
