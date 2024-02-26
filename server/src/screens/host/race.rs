use leptos::*;
use leptos_use::UseIntervalReturn;
use wasm_bindgen::prelude::*;

use crate::utils::use_events;
use shared::models::{monsters, projections};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "sendGameEvent")]
    pub fn send_game_event();
}

#[component]
pub fn race() -> impl IntoView {
    let events = use_events();

    let monsters = projections::monsters(&events.get_untracked());
    let race_seed = projections::race_seed(&events.get_untracked());

    let race = monsters::race(&monsters, race_seed);

    let UseIntervalReturn { counter, .. } = leptos_use::use_interval(1000);

    send_game_event();

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

