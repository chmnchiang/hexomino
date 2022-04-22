use anyhow::{bail, Result};
use gloo::net::http::Request;
use hexomino_api::{AuthPayload, AuthResponse};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

#[derive(Default)]
pub struct LoginModal {
    username: NodeRef,
    password: NodeRef,
    error: Option<String>,
}

pub enum LoginMsg {
    Login,
    LoginFailed(anyhow::Error),
}

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub callback: Callback<AuthResponse>,
}

impl Component for LoginModal {
    type Message = LoginMsg;
    type Properties = LoginProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LoginMsg::Login => {
                let username = self.username.cast::<HtmlInputElement>().unwrap().value();
                let password = self.password.cast::<HtmlInputElement>().unwrap().value();
                let callback_ok = ctx.props().callback.clone();
                let callback_err = ctx.link().callback_once(LoginMsg::LoginFailed);
                let future = async move {
                    match login(AuthPayload { username, password }).await {
                        Ok(auth) => callback_ok.emit(auth),
                        Err(err) => callback_err.emit(err),
                    }
                };
                spawn_local(future);
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
            <div class="modal is-active">
                <div class="modal-background"></div>
                <div class="modal-content">
                    <div style="background-color: white; border-radius: 6px; padding: 10px;">
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
                                <button class="button is-link" onclick={submit}>{"Login"}</button>
                            </div>
                        </div>
                    </div>
                    {
                        if let Some(ref err) = self.error {
                            html! {
                                <div class="notification is-danger" style="margin-top: 10px">
                                    { format!("Login failed: {err}") }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            </div>
        }
    }
}

async fn login(payload: AuthPayload) -> Result<AuthResponse> {
    let response = Request::post("/api/login")
        .json(&payload)
        .unwrap()
        .send()
        .await?;
    if !response.ok() {
        bail!("{}", response.text().await.unwrap());
    }
    Ok(response.json().await?)
}
