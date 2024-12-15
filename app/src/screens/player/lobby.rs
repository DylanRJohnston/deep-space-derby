use leptos::{either::Either, html::Input, prelude::*};

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

    let name_ref = NodeRef::<Input>::new();

    let ready_player = Action::new(move |_: &()| {
        let name = name_ref.get_untracked().unwrap().value();

        async move {
            server_fn::<ChangeProfile>(game_id, &change_profile::Input { name }).await?;
            server_fn::<ReadyPlayer>(game_id, &()).await
        }
    });

    view! {
        <div class="vertical-stack container full-width full-height justify-center">
            <div class="headroom">
                <div class="header left-aligned">{game_id.to_string()}</div>
            </div>
            {move || {
                if !player().ready {
                    Either::Left(
                        view! {
                            <h1>"Profile"</h1>
                            <div class="profile-image">"Placeholder"</div>

                            <div class="input">
                                <label for="name">"Name:"</label>
                                <input
                                    type="text"
                                    id="name"
                                    name="name"
                                    node_ref=name_ref
                                    prop:value=move || player().name
                                />
                            </div>
                            <button
                                class="button"
                                style="margin-top: 2em;"
                                on:click=move |_| {
                                    ready_player.dispatch(());
                                }
                            >

                                "Ready"
                            </button>
                            <a class="button" href="/play">
                                "Leave"
                            </a>
                        },
                    )
                } else {
                    Either::Right(
                        view! {
                            <h1>"Waiting for other players..."</h1>
                            <div class="profile-image">"Image"</div>

                            <h1>{move || player().name}</h1>
                            <span>"Ready"</span>
                        },
                    )
                }
            }}

        </div>
    }
}
