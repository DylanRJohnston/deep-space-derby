use leptos::{component, create_action, create_node_ref, html::Input, view, IntoView, SignalGet};

use crate::{
    server_fns::server_fn,
    utils::{use_events, use_game_id, use_session_id},
};
use shared::models::{
    commands::{change_profile, ChangeProfile, ReadyPlayer},
    projections,
};

#[component]
pub fn lobby() -> impl IntoView {
    let events = use_events();
    let game_id = use_game_id();
    let session_id = use_session_id();

    let players = move || projections::players(&events.get());

    let player = move || players().get(&session_id).cloned().unwrap_or_default();

    let name_ref = create_node_ref::<Input>();

    let ready_player = create_action(move |_: &()| async move {
        server_fn::<ChangeProfile>(
            game_id,
            &change_profile::Input {
                name: name_ref.get_untracked().unwrap().value(),
            },
        )
        .await?;
        server_fn::<ReadyPlayer>(game_id, &()).await
    });

    view! {
        <div class="vertical-stack container full-width full-height justify-center">
            <div class="headroom">
                <div class="header left-aligned">"Lobby Code: " {game_id.to_string()}</div>
            </div>
            {move || {
                if !player().ready {
                    view! {
                        <h1>"Profile"</h1>
                        <div class="profile-image">"Image"</div>

                        <div class="vertical-stack container input">
                            <label for="name">"Name:"</label>
                            <input
                                type="text"
                                id="name"
                                name="name"
                                _ref=name_ref
                                prop:value=move || player().name
                            />
                        </div>
                        <button class="button" on:click=move |_| { ready_player.dispatch(()) }>
                            "Ready"
                        </button>
                        <a class="button" href="/play">
                            "Leave"
                        </a>
                    }
                        .into_view()
                } else {
                    view! {
                        <span>"Waiting for other players..."</span>
                        <div class="profile-image">"Image"</div>

                        <h1>{move || player().name}</h1>
                        <span>"Ready"</span>
                    }
                        .into_view()
                }
            }}

        </div>
    }
}
