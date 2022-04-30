use yew::{function_component, html, Callback, Properties};

#[derive(Properties, PartialEq)]
pub struct WsReconnectModalProps {
    pub logout_cb: Callback<()>,
    pub reconnect_cb: Callback<()>,
}

#[function_component(WsReconnectModal)]
pub fn ws_reconnect_modal(props: &WsReconnectModalProps) -> Html {
    let logout_onclick = props.logout_cb.reform(|_| ());
    let reconnect_onclick = props.reconnect_cb.reform(|_| ());
    html! {
        <div class="modal is-active">
            <div class="modal-background"></div>
            <div class="modal-content">
                <div class="box">
                    <h3 class="subtitle">{"Connection with server lost"}</h3>
                    <div class="field is-grouped">
                        <p class="control">
                            <button class="button is-danger" onclick={logout_onclick}>{"Logout"}</button>
                        </p>
                        <p class="control">
                            <button class="button is-primary" onclick={reconnect_onclick}>{"Reconnect"}</button>
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}
