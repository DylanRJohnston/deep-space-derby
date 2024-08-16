use leptos::*;
use shared::models::projections;

use crate::utils::{use_events, use_game_id};

#[component]
pub fn lobby() -> impl IntoView {
    let game_id = use_game_id();
    let events = use_events();

    let location = window().location();
    let host = location.host().unwrap();

    let url = format!("https://{host}/play?code={game_id}");
    // let url = format!("https://192.168.2.1:8788/play?code={game_id}");
    let url = leptos_router::escape(&url);
    let url = format!(
        "https://api.qrserver.com/v1/create-qr-code/?size=500x500&data={url}&color=fff&bgcolor=000"
    );

    let players = move || projections::players(&events());

    view! {
        <div class="host-lobby-container">
            <img src=url alt=""/>
            <div style="grid-column: 1; grid-row: 2 / span 4;"></div>
            {move || {
                players()
                    .iter()
                    .map(|(_, player)| {
                        view! {
                            <div class="host-lobby-player">
                                <div class="profile-image">"Profile Image"</div>
                                <h1>{player.name.clone()}</h1>
                                <p>{if player.ready { "Ready" } else { "Busy" }}</p>
                            </div>
                        }
                    })
                    .collect_view()
            }}

        </div>
    }
}
