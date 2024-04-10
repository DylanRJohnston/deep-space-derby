use leptos::*;
use shared::models::{events::SceneEvent, monsters::Monster, projections};

use crate::utils::{send_game_event, use_events};

#[component]
pub fn leaderboard() -> impl IntoView {
    let events = use_events();

    let account_balances = move || {
        let events = events();
        let players = projections::players(&events);

        let mut accounts = projections::account_balance(&events)
            .into_iter()
            .filter_map(|(id, balance)| Some((players.get(&id).cloned()?.name, balance)))
            .collect::<Vec<_>>();

        accounts.sort_by(|(_, a), (_, b)| a.cmp(b));
        accounts.into_iter().enumerate().collect::<Vec<_>>()
    };

    view! {
        <div class="leaderboard">
            <span>"Leaderboard: "</span>
            <For each=account_balances key=|it| it.clone() let:data>
                <span>{data.0}</span>
                <span>{data.1}</span>
            </For>
        </div>
    }
}

#[component]
pub fn monster_card(monster: &'static Monster) -> impl IntoView {
    view! {
        <div class="container vertical-stack bg-white">
            <span>{monster.name}</span>
            <div class="avatar-img">"Image"</div>
            <div class="monster-stats">
                <div class="monster-stats">"Speed: " {monster.speed}</div>
            </div>
        </div>
    }
}

#[component]
pub fn pre_game() -> impl IntoView {
    let events = use_events();
    let monsters = move || projections::monsters(projections::race_seed(&events()));

    send_game_event(SceneEvent::PreGame {
        seed: projections::race_seed(&events.get_untracked()),
    });

    view! {
        <div class="vertical-stack full-width full-height container">
            <h1>"Race Overview"</h1>
            <div class="monster-grid">
                <For
                    each=monsters
                    key=|it| (*it).clone()
                    children=|monster| view! { <MonsterCard monster/> }
                />
            </div>
            <span>"Race will start once all bets are placed"</span>
        </div>
    }
}
