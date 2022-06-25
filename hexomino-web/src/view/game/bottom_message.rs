use yew::{function_component, html, use_state, Callback, Children, Properties};

#[derive(Properties, PartialEq)]
pub struct BottomMessageProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(BottomMessage)]
pub fn bottom_message(props: &BottomMessageProps) -> Html {
    let is_collapsed = use_state(|| true);
    let collapse_onclick = {
        let is_collapsed = is_collapsed.clone();
        Callback::from(move |_| is_collapsed.set(!*is_collapsed))
    };

    if *is_collapsed {
        html! {
            <>
                <div style="position: fixed; bottom: 20px; left: 0px; width: 100%;
                    padding-left: 20px; padding-right: 20px; z-index: 10;">
                    <article class="message is-info">
                        <div class="message-body">
                            <div style="display: flex; width: 100%">
                                <div style="flex: 1;">{ for props.children.iter() }</div>
                                <div style="display: flex; width: 40px; align-items: center;">
                                <button class="button is-info" onclick={collapse_onclick.clone()}>
                                    <span class="icon is-small">
                                        <i class="fa-solid fa-angle-right"></i>
                                    </span>
                                </button>
                                </div>
                            </div>
                        </div>
                    </article>
                </div>
                <div style="visibility: hidden; margin-bottom: 10px;">
                    <article class="message is-info">
                        <div class="message-body"> {
                            for props.children.iter()
                        } </div>
                    </article>
                </div>
            </>
        }
    } else {
        html! {
            <>
                <div style="position: fixed; bottom: 20px; right: 20px; display: inline-block;">
                    <button class="button is-info" onclick={collapse_onclick}>
                        <span class="icon">
                            <i class="fa-solid fa-circle-question"></i>
                        </span>
                        <span> {"Help"} </span>
                    </button>
                </div>
                <div style="visibility: hidden; margin-bottom: 20px;">
                </div>
            </>
        }
    }
}
