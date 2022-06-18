use api::{ListUserMatchHistoriesApi, MatchHistoryNoGames};
use wasm_bindgen_futures::spawn_local;
use yew::{
    function_component, html, use_context, use_effect, use_effect_with_deps, use_state, Html,
};

use crate::{context::MainContext, util::ResultExt};

#[function_component(MatchHistoryView)]
pub fn match_history_view(_props: &()) -> Html {
    let match_histories = use_state(|| Vec::<MatchHistoryNoGames>::new());
    let context = use_context::<MainContext>().expect("cannot get context");

    {
        let match_histories = match_histories.clone();
        let connection = context.connection();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    let Ok(result) = connection
                    .get_api::<ListUserMatchHistoriesApi>("/api/match_history/user_list")
                    .await.log_err() else { return; };
                    log::debug!("match histories = {:?}", result);
                    match_histories.set(result);
                });

                || ()
            },
            (),
        );
    }

    fn history_to_row(
        MatchHistoryNoGames {
            id,
            users,
            user_is_first,
            scores,
            end_time,
        }: &MatchHistoryNoGames,
    ) -> Html {
        let user_0_win = scores[0] > scores[1];
        let user_1_win = scores[1] > scores[0];
        html! {
            <tr>
                <td style="text-align: right" class={user_is_first.then_some("my-score")}>
                    if user_0_win {
                        <span class="icon"><i class="fas fa-trophy"></i></span>
                    }
                    <span>{users[0].clone()}</span>
                </td>
                <td class={user_0_win.then_some("score-winner")}>{scores[0]}</td>
                <td>{"-"}</td>
                <td class={user_1_win.then_some("score-winner")}>{scores[1]}</td>
                <td style="text-align: left" class={(!user_is_first).then_some("my-score")}>
                    <span>{users[1].clone()}</span>
                    if user_1_win {
                        <span class="icon"><i class="fas fa-trophy"></i></span>
                    }
                </td>
                <td>{end_time.format("%F %R")}</td>
            </tr>
        }
    }

    html! {
        <div class="columns is-centered">
            <div class="column is-narrow">
                <table class="table">
                    <thead>
                        <tr>
                            <th>{"User 1"}</th>
                            <th style="width: 0%"></th>
                            <th style="width: 0%"></th>
                            <th style="width: 0%"></th>
                            <th>{"User 2"}</th>
                            <th>{"Match end time"}</th>
                        </tr>
                    </thead>
                    <tbody> {
                        match_histories.iter().map(history_to_row).collect::<Html>()
                    } </tbody>
                </table>
            </div>
        </div>
    }
}
