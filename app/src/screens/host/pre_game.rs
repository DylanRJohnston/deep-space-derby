use leptos::*;
use leptos_use::{use_interval, UseIntervalReturn};
use shared::models::{events::OddsExt, monsters::Monster, projections};

use crate::utils::use_events;

#[derive(Debug, PartialEq, Copy, Clone)]
enum Stat {
    Speed,
    Strength,
}

impl Stat {
    fn name(&self) -> &'static str {
        match self {
            Stat::Speed => "Speed",
            Stat::Strength => "Strength",
        }
    }
}

#[component]
fn stat_row(stat: Stat, value: i32) -> impl IntoView {
    view! {
        <div class="monster-stats-row">
            <p class:font-speed=stat == Stat::Speed class:font-strength=stat == Stat::Strength>
                {stat.name()}
                ":"
            </p>
            <div class="stat-bar-container">

                {(1..=value)
                    .map(|i| {
                        view! {
                            <div
                                class="stat-notch"
                                class:stat-speed=stat == Stat::Speed
                                class:stat-strength=stat == Stat::Strength
                            >
                                {if i == value { value.into_view() } else { "".into_view() }}
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}

            </div>
        </div>
    }
}

struct MonsterData {
    monster: Monster,
    odds: f32,
    payout: f32,
}

#[component]
pub fn pre_game() -> impl IntoView {
    let events = use_events();

    let race_seed = move || projections::race_seed(&events());
    let monsters = move || projections::monsters(&events(), race_seed());

    let odds = move || projections::pre_computed_odds(&events());

    let monsters = move || {
        let odds = odds();

        monsters()
            .into_iter()
            .map(|monster| MonsterData {
                monster,
                odds: odds.odds(monster.uuid) * 100.0,
                payout: odds.payout(monster.uuid),
            })
            .collect::<Vec<_>>()
    };

    let UseIntervalReturn { counter, .. } = use_interval(1000);

    let time = move || {
        counter();
        projections::time_left_in_pregame(&events())
    };

    view! {
        <div class="host-pre-game-container">
            <div class="host-pre-game-timer">
                "Time Left: "
                {move || match time() {
                    Some(time) => format!("{time}s"),
                    None => "∞".to_string(),
                }}
            </div>
            <For each=monsters key=|it| it.monster.uuid let:data>
                <div class="monster-stats-container">
                    <h1>{data.monster.name}</h1>
                    <div class="monster-stats-row space-between">
                        <p>"Odds: " {format!("{:.0}", data.odds)} "%"</p>
                        <p>"Payout: " {format!("{:.2}", data.payout)} "x"</p>
                    </div>
                    <StatRow stat=Stat::Speed value=data.monster.speed/>
                    <StatRow stat=Stat::Strength value=data.monster.strength/>
                </div>
            </For>
        </div>
    }
}
