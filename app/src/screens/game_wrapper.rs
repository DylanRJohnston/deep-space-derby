use leptos::{component, create_memo, view, ChildrenFn, IntoView, SignalGet};
use leptos_router::{use_navigate, NavigateOptions};

use crate::utils::{
    create_event_signal, get_session_id, provide_events, provide_session_id, use_game_id,
    Connection,
};

// Turn this into a resource and add an error boundary
#[component]
pub fn game_connection_wrapper(children: ChildrenFn) -> impl IntoView {
    let game_id = use_game_id();

    let (connection, events) = create_event_signal(game_id);

    let connection = create_memo(move |_| connection.get());

    (move || match connection.get() {
        Connection::Connecting => view! { <h1>"Connecting to server"</h1> }.into_view(),
        Connection::Errored(err) => {
            view! { <h1>"Error encountered with connection to server: " {err.to_string()}</h1> }
                .into_view()
        }
        Connection::Closed => view! { <h1>"Connection to server closed"</h1> }.into_view(),
        Connection::Connected => {
            let session_id = get_session_id();
            if session_id.is_none() {
                use_navigate()("/", NavigateOptions::default());

                return ().into_view();
            }

            provide_session_id(session_id.unwrap());
            provide_events(events.into());

            children().into_view()
        }
    })
    .into_view()
}