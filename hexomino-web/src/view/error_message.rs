use std::time::Duration;

use gloo::timers::callback::Timeout;

use yew::{
    function_component, html, use_context, use_effect_with_deps, use_state, Callback,
};

use crate::context::MainContext;

const ERROR_MESSAGE_TIMEOUT: Duration = Duration::from_secs(6);

#[function_component(ErrorMessageView)]
pub fn error_message_view(_props: &()) -> Html {
    let error = use_state(|| -> Option<String> { None });
    let timeout = use_state(|| -> Option<Timeout> { None });
    let callback = {
        let error = error.clone();
        Callback::from(move |err| {
            {
                let error = error.clone();
                timeout.set(Some(Timeout::new(
                    ERROR_MESSAGE_TIMEOUT.as_millis() as u32,
                    move || error.set(None),
                )))
            }
            error.set(Some(err))
        })
    };
    let context = use_context::<MainContext>();

    use_effect_with_deps(
        move |context| {
            if let Some(context) = context {
                context.main().set_error_message_callback(callback)
            }
            || ()
        },
        context,
    );

    let delete_onclick = {
        let error = error.clone();
        Callback::from(move |_| error.set(None))
    };

    if let Some(error) = &*error {
        html! {
            <div style="position: fixed; bottom: 20px; left: 50%; width: 400px; margin-left: -200px; z-index: 100;"
                class="notification is-danger">
                <button class="delete" onclick={delete_onclick}></button>
                { error }
            </div>
        }
    } else {
        html! {}
    }
}
