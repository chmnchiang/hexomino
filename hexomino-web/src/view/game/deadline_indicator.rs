use api::Deadline;
use chrono::Utc;
use gloo::render::{request_animation_frame, AnimationFrame};
use yew::{classes, function_component, html, use_effect, use_mut_ref, use_state, Properties};

#[derive(PartialEq, Properties)]
pub struct DeadlineIndicatorProps {
    pub deadline: Deadline,
}

#[function_component(DeadlineIndicator)]
pub fn deadline_indicator(props: &DeadlineIndicatorProps) -> Html {
    let current_time = use_state(|| Utc::now());

    let animation_frame_handle = use_mut_ref::<Option<AnimationFrame>>(|| None);
    {
        let current_time = current_time.clone();
        use_effect(move || {
            *animation_frame_handle.borrow_mut() = Some(request_animation_frame(move |_| {
                current_time.set(Utc::now())
            }));

            || ()
        });
    }

    let time_left_millisec = (props.deadline.time - *current_time).num_milliseconds();
    let duration_millisec = props.deadline.duration.as_millis() as i64;
    let percentage =
        time_left_millisec.clamp(0, duration_millisec) as f64 / duration_millisec as f64;

    let color = if percentage >= 0.5 {
        "is-success"
    } else if percentage >= 0.2 {
        "is-warning"
    } else {
        "is-danger"
    };

    html! {
        <div class="columns is-mobile is-centered">
            <div class="column is-full">
                <progress class={classes!("progress", color)} max="1000"
                    value={((percentage * 1000.0) as i64).to_string()}></progress>
            </div>
        </div>
    }
}
