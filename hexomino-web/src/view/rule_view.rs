use yew::{function_component, html};

#[function_component(RuleView)]
pub fn rule_view(_props: &()) -> Html {
    html! {
        <div class="columns is-centered">
            <div class="column is-three-quarters">
                <div class="content">
                    <h1> {"Game Rules"} </h1>
                    <p>
                    {"A "} <b>{"hexomino"}</b> {r#" is a polygon formed by 6 equal-sized squares connecting edge-to-edge.
                        This game is played using 34 hexominos, which include of all possible hexominos
                        (when rotations and reflections are considered to be the same) except the one with the I shape."#}
                    </p>
                    <p>
                        {"The game has two phases: the "}
                        <b>{"Pick Phase"}</b>
                        {" and the "}
                        <b>{"Place Phase"}</b>
                        {"."}
                    </p>
                    <ol>
                        <li>
                            <b>{"Pick Phase: "}</b>
                            {r#"During the pick phase, two players take turns to take one hexomino that hasn't been picked previously
                                and add in to the player's inventory, until all 34 hexominos have been taken by a player.
                                There is a fixed time limit of "#} <b>{"15 seconds"}</b> {r#" (except for AI games,
                                which are meant for practicing and so have no time limit imposed)
                                for picking a hexomino. If the time limit is reached and no hexomino is picked by a
                                player, "#} <b>{"a random hexomino will be picked"}</b> {" for that player."}
                        </li>
                        <li>
                            <b>{"Place Phase: "}</b>
                            <p>{r#"During the place phase, two players take turns placing one hexomino from the player's inventory
                                on the board. The player that goes second in the pick phase will go first in this phase.
                                The playing board is a 12×18 grid. The hexomino must be placed so that all six squares
                                are on empty tiles of the board. A tile on the board is empty if it is not occupied by
                                other hexominos. The game ends and a winner is declared when one of the following happens:"#}</p>
                            <ul>
                                <li>{"A player cannot place a hexomino. This includes the case when there are no positions
                                    on the board for the player to place or when the player has no hexomino in their inventory."}</li>
                                <li>{"The time limit exceeds. A player must find a place to put one of their hexomino before the time limit.
                                    The time limit could differ from different match settings."}</li>
                            </ul>
                            <p>{r#"The system will declare the winner immediately once any of the two conditions has been met.
                                That is, if the game keeps going, there must be a place to put a hexomino without collision.
                                    Yet, you need to figure out where to place before the time limit!"#} </p>
                        </li>
                    </ol>
                    <p> {"There will be one or more games in each match. The player who goes first
                        in the pick phase will be determined randomly. In the subsequent matches,
                        The player that goes second in the previous game will be the first."} </p>
                </div>
            </div>
        </div>
    }
}
