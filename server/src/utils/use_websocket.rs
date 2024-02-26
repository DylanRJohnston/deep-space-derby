use im::vector::Vector;
use leptos::{provide_context, use_context, ReadSignal, ServerFnError};
use leptos_reactive::{Signal, SignalUpdate};
use shared::models::events::Event;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Connection {
    Connecting,
    Connected,
    Errored(ServerFnError),
    Closed,
}

#[cfg(not(feature = "ssr"))]
async fn sleep(ms: i32) {
    use js_sys::{Function, Promise};
    use wasm_bindgen::{closure::Closure, JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    let promise = Promise::new(&mut |resolve: Function, _: Function| {
        let closure = Closure::<dyn FnMut()>::new(move || {
            resolve.call0(&JsValue::UNDEFINED).unwrap();
        });

        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.into_js_value().unchecked_ref(),
                ms,
            )
            .unwrap();
    });

    JsFuture::from(promise).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn create_event_signal(url: String) -> (ReadSignal<Connection>, ReadSignal<Vector<Event>>) {
    use futures_util::StreamExt;
    use gloo_net::websocket::{futures::WebSocket, Message};
    use leptos::*;
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

                leptos::logging::log!("Event {:#?}", &event);

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

#[cfg(feature = "ssr")]
#[allow(unused_variables)]
pub fn create_event_signal(url: String) -> (ReadSignal<Connection>, ReadSignal<Vector<Event>>) {
    use leptos::create_signal;

    let (connection, _) = create_signal(Connection::Connecting);
    let (events, _) = create_signal(Vector::new());

    (connection, events)
}

#[derive(Clone)]
struct EventsContainer(Signal<Vector<Event>>);

pub fn provide_events(signal: Signal<Vector<Event>>) {
    provide_context(EventsContainer(signal));
}

pub fn use_events() -> Signal<Vector<Event>> {
    use_context::<EventsContainer>().unwrap().0
}


