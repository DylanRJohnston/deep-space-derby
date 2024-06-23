use leptos::*;
use leptos_use::UseIntervalReturn;

use crate::utils::use_events;
use shared::models::{monsters, projections};

#[component]
pub fn race() -> impl IntoView {
    let events = use_events();

    let seed = projections::race_seed(&events.get_untracked());
    let monsters = projections::monsters(seed);

    let _race = monsters::race(&monsters, seed);

    let UseIntervalReturn { .. } = leptos_use::use_interval(1000);

    view! {
        <div class="race-container">
            <h1>"Race!"</h1>
            <div class="racecourse">

                {monsters
                    .iter()
                    .map(|monster| view! { <div>{monster.name}</div> })
                    .collect::<Vec<_>>()
                    .into_view()}

            </div>
        </div>
    }
}
