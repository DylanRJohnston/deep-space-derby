use leptos::{component, create_action, create_effect, view, IntoView, ServerFnError, SignalGet};
use leptos_router::{use_navigate, NavigateOptions};

use crate::{
    models::commands::{create_game, CreateGame},
    server_fns::server_fn,
    utils::generate_game_code,
};

#[component]
pub fn main_menu() -> impl IntoView {
    let game_code: leptos::Action<(), Result<String, ServerFnError>> =
        create_action(|_: &()| async move {
            let game_id = generate_game_code();

            server_fn::<CreateGame>(
                &game_id,
                &create_game::Input {
                    code: game_id.clone(),
                },
            )
            .await?;

            Ok(game_id)
        });

    let button_text = move || {
        if game_code.pending().get() {
            "Creating Game"
        } else {
            "Host"
        }
    };

    let navigate = use_navigate();

    create_effect(move |_| {
        if let Some(Ok(game_code)) = game_code.value().get() {
            navigate(
                &format!("/host/{}/lobby", game_code),
                NavigateOptions::default(),
            );
        }
    });

    view! {
        <div class="main-menu">
            <h1 class="title">"Deep Space Derby"</h1>
            <button class="host" on:click=move |_| game_code.dispatch(()) >
                <h2>{button_text}</h2>
            </button>
            <button class="play">
                <a href="/play">
                    <h2>"Play"</h2>
                </a>
            </button>
            <button class="exit">
                <h2>"Exit"</h2>
            </button>
        </div>
    }
}
