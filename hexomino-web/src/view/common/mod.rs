use api::MatchToken;
use yew::{Html, html, classes};

pub mod hexo_svg;

pub fn match_token_html(match_token: &Option<MatchToken>, is_large: bool) -> Html {
    let is_large_class = is_large.then_some("is-medium");
    if let Some(token) = match_token {
        html! {
            <div class={classes!("tags", "has-addons", is_large_class)}>
                <span class="tag is-dark">{"match"}</span>
                <span class="tag is-info">{token.0.clone()}</span>
            </div>
        }
    } else {
        html! { <span class={classes!("tag", is_large_class)}>{"normal"}</span> }
    }
}
