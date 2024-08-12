use leptos::*;
use leptos_use::*;
use shared::models::projections;

use crate::utils::use_events;

#[component]
pub fn wait() -> impl IntoView {
    let events = use_events();

    let UseIntervalReturn { counter, .. } = use_interval(1000);

    let time = move || {
        counter();
        projections::time_left_in_pregame(&events())
    };

    view! {
        <div class="pre-game-container space-around">
            <h1>"waiting for other players..."</h1>
            <div class="countdown">{time}</div>
            <h2>"Ready"</h2>
        </div>
    }
}
