use im::vector::Vector;
use leptos::prelude::*;
use shared::models::{events::Event, game_code::GameCode};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Connection {
    Connecting,
    Connected,
    Reconnecting,
    Errored,
    Closed,
}

#[cfg(feature = "hydrate")]
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

#[cfg(feature = "hydrate")]
pub fn create_event_signal(
    game_id: GameCode,
) -> (ReadSignal<Connection>, ReadSignal<Vector<Event>>) {
    use futures_util::StreamExt;
    use gloo_net::websocket::{futures::WebSocket, Message};
    use leptos::{server_fn::error::NoCustomError, *};
    use shared::models::events::EventStream;
    use wasm_bindgen_futures::spawn_local;

    let url = {
        let location = window().location();
        let host = location.host().unwrap();

        format!("wss://{}/api/object/game/by_code/{}/connect", host, game_id)
    };

    let (connection, set_connection) = signal(Connection::Connecting);
    let (events, set_events) = signal(Vector::new());

    spawn_local(async move {
        let mut count = 0;
        loop {
            let result: Result<(), ServerFnError> = try {
                let mut socket = WebSocket::open(&url)?;

                set_events(Vector::new());

                while let Some(msg) = socket.next().await {
                    count = 0;

                    let event = match msg {
                        Ok(Message::Text(text)) => serde_json::from_str::<EventStream>(&text)
                            .map_err(|err: serde_json::Error| {
                                ServerFnError::<NoCustomError>::Deserialization(err.to_string())
                            }),
                        Ok(Message::Bytes(_)) => Err(ServerFnError::Deserialization(
                            "got binary message on websocket".into(),
                        )),
                        Err(err) => Err(ServerFnError::ServerError(err.to_string())),
                    }?;

                    match event {
                        EventStream::Events(new_events) => {
                            tracing::info!(?new_events, "got new events");
                            set_events.update(|events| events.append(new_events.into()))
                        }
                        EventStream::Event(event) => {
                            tracing::info!(?event, "got new event");
                            set_events.update(|events| events.push_back(event))
                        }
                    }

                    set_connection.set(Connection::Connected);
                }
            };

            match result {
                Ok(_) => {
                    set_connection.set(Connection::Closed);
                    break;
                }
                Err(err) => {
                    tracing::error!(?err);
                    set_connection.set(Connection::Reconnecting)
                }
            }

            count += 1;

            sleep(count * 1000).await;

            if count > 5 {
                set_connection.set(Connection::Errored);
                break;
            }
        }
    });

    (connection, events)
}

#[cfg(feature = "ssr")]
#[allow(unused_variables)]
pub fn create_event_signal(
    game_id: GameCode,
) -> (ReadSignal<Connection>, ReadSignal<Vector<Event>>) {
    use leptos::prelude::*;

    let (connection, _) = signal(Connection::Connecting);
    let (events, _) = signal(Vector::new());

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
