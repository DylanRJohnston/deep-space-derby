use leptos::{component, view, IntoView, SignalGet};

use crate::{
    models::projections,
    utils::{use_events, use_game_id},
};

#[component]
pub fn player_lobby() -> impl IntoView {
    let events = use_events();
    let game_id = use_game_id();

    let player_count = move || projections::player_count(&events.get());

    view! {
        <div>
            <h1>"Welcome player to "{game_id}</h1>
            <h2>{player_count}" players connected"</h2>
        </div>
    }
}
