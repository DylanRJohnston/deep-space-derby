use leptos::{either::Either, prelude::*};
use leptos_use::{use_interval, UseIntervalReturn};
use shared::models::{events::OddsExt, monsters::Monster, projections};

use crate::utils::use_events;

#[derive(Debug, PartialEq, Copy, Clone)]
enum Stat {
    Dexterity,
    Strength,
}

impl Stat {
    fn name(&self) -> &'static str {
        match self {
            Stat::Dexterity => "Dexterity",
            Stat::Strength => "Strength",
        }
    }
}

#[component]
fn stat_row(stat: Stat, value: i32) -> impl IntoView {
    view! {
        <p
            class="monster-stats-row"
            class:font-dexterity=stat == Stat::Dexterity
            class:font-strength=stat == Stat::Strength
        >
            {stat.name()}
            ":"
        </p>
        <div class="stat-bar-container">

            {(1..=value)
                .map(|i| {
                    view! {
                        <div
                            class="stat-notch"
                            class:stat-dexterity=stat == Stat::Dexterity
                            class:stat-strength=stat == Stat::Strength
                        >
                            {if i == value {
                                Either::Left(value.into_view())
                            } else {
                                Either::Right("".into_view())
                            }}

                        </div>
                    }
                })
                .collect::<Vec<_>>()}

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

    let round_number = move || projections::round(&events());

    view! {
        <div class="host-pre-game-container">
            <div class="host-pre-game-timer" style="left: 1em;">
                "Round "
                {round_number}
                " of 10"
            </div>
            <div class="host-pre-game-timer" style="justify-self: center;">
                "Time Left "
                {move || match time() {
                    Some(time) => format!("{time}s"),
                    None => "∞".to_string(),
                }}

            </div>
            <For each=monsters key=|it| it.monster.uuid let:data>
                <div class="monster-stats-container">
                    <h1>{data.monster.name}</h1>
                    <p class="odds">"Odds: "</p>
                    <p class="odds" style="text-align: left;">
                        {format!("{:.0}", data.odds)}
                        "%"
                    </p>
                    <StatRow stat=Stat::Dexterity value=data.monster.dexterity/>
                    <StatRow stat=Stat::Strength value=data.monster.strength/>
                </div>
            </For>
        </div>
    }
}
