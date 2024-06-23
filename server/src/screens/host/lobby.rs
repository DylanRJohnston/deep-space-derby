use leptos::*;

use crate::utils::{use_events, use_game_id};
use shared::models::projections;

#[component]
fn player(name: String, ready: bool) -> impl IntoView {
    view! {
        <div class="container bg-white vertical-stack">
            <div class="avatar-img">"Image"</div>
            <span>{name}</span>
            {move || if ready { "Ready" } else { "Busy..." }}
        </div>
    }
}

#[component]
pub fn lobby() -> impl IntoView {
    let events = use_events();
    let game_id = use_game_id();

    let players = move || projections::players(&events.get());

    view! {
        <div class="vertical-stack container full-height full-width">
            <h1>"Lobby Code = " {game_id.to_string()}</h1>
            <div class="avatar-grid">
                <For
                    each=players
                    key=|it| it.1.clone()
                    children=move |(_, player)| {
                        view! { <Player name=player.name ready=player.ready/> }
                    }
                />

            </div>
        </div>
    }
}
