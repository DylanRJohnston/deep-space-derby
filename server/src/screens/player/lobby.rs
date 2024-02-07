use leptos::{component, create_action, create_node_ref, html::Input, view, IntoView, SignalGet};

use crate::{
    models::{
        commands::{change_profile, ChangeProfile, ReadyPlayer},
        projections,
    },
    server_fns::server_fn,
    utils::{use_events, use_game_id, use_session_id},
};

#[component]
pub fn lobby() -> impl IntoView {
    let events = use_events();
    let game_id = use_game_id();
    let session_id = use_session_id();

    let players = move || projections::players(&events.get());
    let player_count = move || players().len();
    let ready_count = move || {
        players()
            .into_iter()
            .filter_map(|(_, info)| info.ready.then_some(true))
            .count()
    };

    let player = move || players().get(&session_id).cloned().unwrap_or_default();

    let name_ref = create_node_ref::<Input>();

    let change_profile = {
        let game_id = game_id.clone();

        create_action(move |name: &String| {
            server_fn::<ChangeProfile>(
                &game_id,
                &change_profile::Input {
                    name: name.to_owned(),
                },
            )
        })
    };

    let ready_player = {
        let game_id = game_id.clone();

        create_action(move |_: &()| server_fn::<ReadyPlayer>(&game_id, &()))
    };

    view! {
        <div class="player-lobby-container">
            <div class="top-row">
                <div>"Lobby: "{game_id}</div>
                <div>"Ready: "{ready_count}"/"{player_count}</div>
            </div>
            <img class="profile-picture" src="https://upload.wikimedia.org/wikipedia/commons/8/89/Portrait_Placeholder.png" />
            <div class="name-change">
                <p>"Name:"</p>
                <input type="text" _ref=name_ref prop:value=move || player().name />
                <button on:click=move |_| change_profile.dispatch(name_ref.get().unwrap().value()) >"Change"</button>
            </div>
            <div class="button-tray">
                <button>"Leave"</button>
                <button on:click=move |_| ready_player.dispatch(())>"Ready"</button>
            </div>
        </div>
    }
}
