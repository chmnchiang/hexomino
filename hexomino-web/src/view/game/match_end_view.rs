use api::{MatchEndInfo, MatchWinner};
use yew::{function_component, html, use_state, Callback, Children, Properties};

#[derive(Properties)]
pub struct MatchEndProps {
    pub info: MatchEndInfo,
    pub names: [String; 2],
}

impl PartialEq for MatchEndProps {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

#[function_component(MatchEndView)]
pub fn match_end_view(props: &MatchEndProps) -> Html {
    let names = &props.names;
    let MatchEndInfo { scores, winner } = props.info;
    html! {
        <>
            <div class="columns is-centered">
                <div class="column is-half" style="text-align: center">
                    <h2 class="title is-2">
                    { "Match Ended" }
                    </h2>
                </div>
            </div>
            <div class="columns is-centered">
                <div class="column is-one-quarter" style="text-align: center">
                    <div class="card" style="text-align: center">
                        <div class="card-content">
                            <div class="content">
                                <h2 class="title is-3 my-foreground">
                                { names[0].clone() }
                                </h2>
                                <h2 class="title is-2">
                                { scores[0] }
                                </h2>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="column is-one-quarter" style="text-align: center">
                    <div class="card" style="text-align: center">
                        <div class="card-content">
                            <div class="content">
                                <h2 class="title is-3 their-foreground">
                                { names[1].clone() }
                                </h2>
                                <h2 class="title is-2">
                                { scores[1] }
                                </h2>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <div class="columns is-centered">
                <div class="column is-full" style="text-align: center">
                {
                    match winner {
                        MatchWinner::You => html! {
                            <h2 class="title is-3 my-foreground"> { format!("The winner is {} (You).", names[0].clone()) } </h2>
                        },
                        MatchWinner::They => html! {
                            <h2 class="title is-3 their-foreground"> { format!("The winner is {}.", names[1].clone()) } </h2>
                        },
                        MatchWinner::Tie => html! {
                            <h2 class="title is-3"> { "It's a tie!" } </h2>
                        },
                    }
                }
                </div>
            </div>
        </>
    }
}
