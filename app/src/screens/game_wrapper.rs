use leptos::*;
use leptos_router::{use_navigate, NavigateOptions};

use crate::utils::{
    create_event_signal, get_session_id, provide_events, provide_session_id, use_game_id,
    Connection,
};

// Turn this into a resource and add an error boundary
#[component]
pub fn game_connection_wrapper(#[prop(optional)] children: Option<ChildrenFn>) -> impl IntoView {
    let game_id = use_game_id();

    let (connection, events) = create_event_signal(game_id);

    let connection = create_memo(move |_| connection.get());

    (move || match connection.get() {
        Connection::Connecting => view! {
            <div class="server-status">
                <h1>"Connecting..."</h1>
                <div class="loader"></div>
            </div>
        }
        .into_view(),
        Connection::Errored => view! {
            <div class="server-status">
                <h1>"Error"</h1>
                <h1>"Refresh the page"</h1>
            </div>
        }
        .into_view(),
        Connection::Reconnecting => view! {
            <div class="server-status">
                <h1>"Reconnecting..."</h1>
                <div class="loader"></div>
            </div>
        }
        .into_view(),
        Connection::Closed => view! {
            <div class="server-status">
                <h1>"Refresh the page"</h1>
            </div>
        }
        .into_view(),
        Connection::Connected => {
            let session_id = get_session_id();
            if session_id.is_none() {
                use_navigate()("/", NavigateOptions::default());

                return ().into_view();
            }

            provide_session_id(session_id.unwrap());
            provide_events(events.into());

            children.clone().into_view()
        }
    })
    .into_view()
}
