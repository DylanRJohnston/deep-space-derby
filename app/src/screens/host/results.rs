use leptos::prelude::*;
use shared::models::projections;

use crate::utils::use_events;

struct LeaderboardInfo {
    name: String,
    balance: i32,
    winnings: i32,
}

#[component]
pub fn results() -> impl IntoView {
    let events = use_events();

    let timer = leptos_use::use_timeout_fn(|_| {}, 3000.);
    (timer.start)(());

    let players = move || projections::players(&events());
    let balances = move || projections::all_account_balances(&events());
    let winnings = move || projections::winnings(&events());
    let debt = move || projections::all_debt(&events());

    let leaderboard = move || {
        let players = players();
        let winnings = winnings();
        let balances = balances();
        let debt = debt();

        let mut info = players
            .into_iter()
            .filter_map(|(key, value)| {
                Some(LeaderboardInfo {
                    name: value.name,
                    balance: balances.get(&key).copied().unwrap_or_default()
                        - debt.get(&key).copied().unwrap_or_default() as i32,
                    winnings: winnings.get(&key).copied().unwrap_or_default(),
                })
            })
            .collect::<Vec<_>>();

        info.sort_by(|a, b| a.balance.cmp(&b.balance).reverse());

        info
    };

    let game_is_finished = move || projections::game_finished(&events());

    view! {
        <div class="host-results-container">
            <div
                class="results-container"
                class:invisible=timer.is_pending
                class:two-columns=move || (leaderboard().len() > 7)
            >
                <h1>
                    {move || if game_is_finished() { "Final Score" } else { "Player Leaderboard" }}
                </h1>

                {move || {
                    leaderboard()
                        .into_iter()
                        .enumerate()
                        .map(|(position, info)| {
                            let row = position % 7 + 3;
                            let column = 5 * (position / 7);
                            view! {
                                <div
                                    class="leaderboard-background"
                                    style=format!(
                                        "grid-row: {}; grid-column: {} / span 4",
                                        row,
                                        column + 1,
                                    )
                                ></div>
                                <div
                                    class="leaderboard-entry-position"
                                    style=format!("grid-row: {}; grid-column: {}", row, column + 1)
                                >
                                    {position + 1}
                                    "."
                                </div>
                                <div
                                    class="leaderboard-entry-name"
                                    style=format!("grid-row: {}; grid-column: {}", row, column + 2)
                                >
                                    {info.name}
                                </div>
                                <div
                                    class="leaderboard-entry-balance"
                                    style=format!("grid-row: {}; grid-column: {}", row, column + 3)
                                >
                                    "💎 "
                                    {info.balance}
                                </div>
                                <div
                                    class="leaderboard-entry-winnings"
                                    style=format!("grid-row: {}; grid-column: {}", row, column + 4)
                                >
                                    {format!("💎 {:+}", info.winnings)}
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                }}

            </div>
        </div>
    }
}
