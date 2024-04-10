use leptos::*;
use leptos_use::UseIntervalReturn;
use wasm_bindgen::prelude::*;

use crate::utils::{send_game_event, use_events};
use shared::models::{events::SceneEvent, monsters, projections};

#[component]
pub fn race() -> impl IntoView {
    let events = use_events();

    let seed = projections::race_seed(&events.get_untracked());
    let monsters = projections::monsters(seed);

    let _race = monsters::race(&monsters, seed);

    let UseIntervalReturn { .. } = leptos_use::use_interval(1000);

    send_game_event(SceneEvent::Race { seed });

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
