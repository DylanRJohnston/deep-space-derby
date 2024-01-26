use crate::models::events::Event;

use im::vector::Vector;
use leptos::{provide_context, use_context, ReadSignal, ServerFnError};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Connection {
    Connecting,
    Connected,
    Errored(ServerFnError),
    Closed,
}

// #[cfg(not(feature = "ssr"))]
pub fn create_event_signal(url: String) -> (ReadSignal<Connection>, ReadSignal<Vector<Event>>) {
    use futures_util::StreamExt;
    use gloo_net::websocket::{futures::WebSocket, Message};
    use leptos::{create_signal, leptos_dom::logging::console_log, SignalSet, SignalUpdate};
    use wasm_bindgen_futures::spawn_local;

    let (connection, set_connection) = create_signal(Connection::Connecting);
    let (events, set_events) = create_signal(Vector::new());

    spawn_local(async move {
        let result: Result<(), ServerFnError> = try {
            let mut socket = WebSocket::open(&url)?;

            while let Some(msg) = socket.next().await {
                let event = match msg {
                    Ok(Message::Text(text)) => {
                        serde_json::from_str::<Event>(&text).map_err(|err: serde_json::Error| {
                            ServerFnError::Deserialization(err.to_string())
                        })
                    }
                    Ok(Message::Bytes(_)) => Err(ServerFnError::Deserialization(
                        "got binary message on websocket".into(),
                    )),
                    Err(err) => Err(ServerFnError::ServerError(err.to_string())),
                }?;

                console_log(&format!("Event received {:?}", &event));

                set_connection.set(Connection::Connected);
                set_events.update(|events| events.push_back(event));
            }
        };

        match result {
            Ok(_) => set_connection.set(Connection::Closed),
            Err(err) => set_connection.set(Connection::Errored(err)),
        }
    });

    (connection, events)
}

// #[cfg(feature = "ssr")]
// #[allow(unused_variables)]
// pub fn create_event_signal(url: String) -> (ReadSignal<Connection>, ReadSignal<Vector<Event>>) {
//     use leptos::create_signal;

//     let (connection, _) = create_signal(Connection::Connecting);
//     let (events, _) = create_signal(Vector::new());

//     (connection, events)
// }

#[derive(Clone)]
struct Events(ReadSignal<Vector<Event>>);

pub fn provide_events(signal: ReadSignal<Vector<Event>>) {
    provide_context(Events(signal));
}

pub fn use_events() -> ReadSignal<Vector<Event>> {
    use_context::<Events>().unwrap().0
}
