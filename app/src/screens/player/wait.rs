use leptos::prelude::*;
use leptos_use::*;
use shared::models::projections;

use crate::{
    screens::player::VictimModal,
    utils::{use_events, use_session_id},
};

#[component]
pub fn wait() -> impl IntoView {
    let events = use_events();
    let player_id = use_session_id();

    let UseIntervalReturn { counter, .. } = use_interval(1000);

    let time = move || {
        counter();
        projections::time_left_in_pregame(&events())
    };

    let (victim_modal, set_victim_modal) = signal(None);

    Effect::new(move |_| {
        if let Some(card) = projections::victim_of_card(&events(), player_id) {
            set_victim_modal(Some(card))
        }
    });

    view! {
        <Show when=move || victim_modal().is_none()>
            <div class="pre-game-container space-around">
                <h1>"waiting for other players..."</h1>
                <div class="countdown">
                    {move || match time() {
                        Some(time) => format!("{time}"),
                        None => "∞".to_string(),
                    }}

                </div>
                <h2>"Ready"</h2>
            </div>
        </Show>
        {move || {
            victim_modal()
                .map(|(card, perpetrator)| {
                    view! { <VictimModal card perpetrator close=move || set_victim_modal(None) /> }
                })
        }}
    }
}
