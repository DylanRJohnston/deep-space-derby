use leptos::{component, view, CollectView, IntoView, SignalGet};

use crate::{
    models::projections,
    utils::{use_events, use_game_id},
};

#[component]
pub fn lobby() -> impl IntoView {
    let events = use_events();
    let game_id = use_game_id();

    let players = move || projections::players(&events.get());
    let player_count = move || players().len();
    let ready_count = move || {
        players()
            .into_iter()
            .filter_map(|(_, info)| info.ready.then_some(true))
            .count()
    };

    view! {
        <div class="host-lobby-container">
            <div class="top-row">
                <div>"Lobby: " <span data-testid="game_code">{game_id}</span></div>
                <div>"Ready: " {ready_count} "/" {player_count}</div>
            </div>
            <div class="avatar-previews">

                {move || {
                    players()
                        .values()
                        .map(|info| {
                            view! {
                                <div class="avatar-container">
                                    <img
                                        class="profile-picture"
                                        src="https://upload.wikimedia.org/wikipedia/commons/8/89/Portrait_Placeholder.png"
                                    />
                                    <div class="name">{info.name.clone()}</div>
                                    <div class="status">
                                        "Ready: " {if info.ready { "✅" } else { "❌" }}
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()
                }}

            </div>
        </div>
    }
}

