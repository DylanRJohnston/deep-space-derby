use leptos::{
    component, create_action, create_effect, create_signal, event_target_value, view, IntoView,
    SignalGet, SignalSet,
};
use leptos_router::{use_navigate, NavigateOptions};

use crate::{
    models::commands::{self, join_game},
    server_fns::server_fn,
};

#[component]
pub fn join_screen() -> impl IntoView {
    let (code, set_code_inner) = create_signal("".to_owned());

    let set_code = move |mut value: String| {
        value.truncate(6);
        set_code_inner.set(value.to_uppercase());
    };

    let on_click = create_action(|code: &String| {
        server_fn::<commands::JoinGame>(
            code,
            &join_game::Input {
                name: "Bob".to_owned(),
            },
        )
    });

    let navigate = use_navigate();

    create_effect(move |_| {
        if let Some(Ok(_)) = on_click.value().get() {
            navigate(
                &format!("/play/{}/lobby", code.get()),
                NavigateOptions::default(),
            );
        }
    });

    view! {
        <div class="join-screen">
            <h1>"Join Game"</h1>
            <div class="join-input">
                <p>"Lobby Code"</p>
                <input
                    type="text"
                    name="code"
                    on:input=move |ev| { set_code(event_target_value(&ev)) }
                    prop:value=code
                />
            </div>
            <div class="button-tray">
                <button>
                    <a href="/">"Back"</a>
                </button>
                <input type="submit" on:click=move |_| on_click.dispatch(code.get())/>
            </div>
        </div>
    }
}
