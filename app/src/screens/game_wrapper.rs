use leptos::prelude::*;
use leptos_router::{hooks::use_navigate, NavigateOptions};

use crate::utils::{
    create_event_signal, get_session_id, provide_events, provide_session_id, use_game_id,
    Connection,
};

// Turn this into a resource and add an error boundary
#[component]
pub fn game_connection_wrapper(children: ChildrenFn) -> impl IntoView {
    let game_id = use_game_id();

    let (connection, events) = create_event_signal(game_id);

    let connection = Memo::new(move |_| connection.get());

    move || match connection.get() {
        Connection::Connecting => view! {
            <div class="server-status">
                <h2>"Connecting..."</h2>
                <div class="loader"></div>
            </div>
        }
        .into_any(),
        Connection::Errored => view! {
            <div class="server-status">
                <h2>"Error"</h2>
                <h2>"Refresh the page"</h2>
            </div>
        }
        .into_any(),
        Connection::Reconnecting => view! {
            <div class="server-status">
                <h2>"Reconnecting..."</h2>
                <div class="loader"></div>
            </div>
        }
        .into_any(),
        Connection::Closed => view! {
            <div class="server-status">
                <h2>"Refresh the page"</h2>
            </div>
        }
        .into_any(),
        Connection::Connected => {
            let session_id = get_session_id();
            if session_id.is_none() {
                use_navigate()("/", NavigateOptions::default());

                return ().into_any();
            }

            provide_session_id(session_id.unwrap());
            provide_events(events.into());

            children()
        }
    }
}
