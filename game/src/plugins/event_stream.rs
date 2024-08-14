use std::sync::LazyLock;

use bevy::{prelude::*, utils::tracing};

use crossbeam_channel::{Receiver, Sender};
use im::Vector;
use shared::models::{events::Event, game_id::GameID};

use super::scenes::SceneState;

pub struct EventStreamPlugin;

impl Plugin for EventStreamPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameEvents(Vector::new()))
            .add_systems(Update, read_event_stream)
            .add_systems(Update, reset_event_stream)
            .add_systems(Update, transition_debug);

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Startup, connect_to_server);
    }
}

struct EventChannel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

static EVENT_CHANNEL: LazyLock<EventChannel<Event>> = LazyLock::new(|| {
    let (sender, receiver) = crossbeam_channel::unbounded::<Event>();

    EventChannel { sender, receiver }
});

static RESET_CHANNEL: LazyLock<EventChannel<()>> = LazyLock::new(|| {
    let (sender, receiver) = crossbeam_channel::unbounded::<()>();

    EventChannel { sender, receiver }
});

#[derive(Resource)]
pub struct Seed(pub u32);

#[derive(Resource)]
pub struct GameEvents(pub Vector<Event>);

impl std::ops::Deref for GameEvents {
    type Target = Vector<Event>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn read_event_stream(
    // mut next_state: ResMut<NextState<SceneState>>,
    mut events: ResMut<GameEvents>,
) {
    while let Ok(event) = EVENT_CHANNEL.receiver.try_recv() {
        tracing::info!(?event);
        events.as_mut().0.push_back(event);
    }
}

fn reset_event_stream(mut events: ResMut<GameEvents>) {
    while let Ok(_) = RESET_CHANNEL.receiver.try_recv() {
        tracing::error!("event stream reset");
        events.as_mut().0.clear();
    }
}

// #[cfg(feature = debug)]
fn transition_debug(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<SceneState>>,
    mut next_state: ResMut<NextState<SceneState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(match state.get() {
            SceneState::Lobby => SceneState::PreGame,
            SceneState::PreGame => SceneState::Lobby,
            other => *other,
        })
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(js_name = "sendGameEvent")]
pub fn send_game_event(event: Event) -> Result<(), wasm_bindgen::JsError> {
    EVENT_CHANNEL.sender.send(event).map_err(Into::into)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(js_name = "resetGameEvents")]
pub fn reset_game_events() -> Result<(), wasm_bindgen::JsError> {
    RESET_CHANNEL.sender.send(()).map_err(Into::into)
}

#[derive(Debug, Resource, Deref)]
pub struct GameCode(pub GameID);

#[cfg(not(target_arch = "wasm32"))]
fn connect_to_server(game_code: Res<GameCode>) {
    use anyhow::Context;
    use bevy::tasks::IoTaskPool;
    use tungstenite::Message;

    let (mut socket, _) = tungstenite::connect(format!(
        "ws://localhost:8000/api/object/game/by_code/{code}/connect",
        code = **game_code
    ))
    .context("failed to connect to web socket")
    .unwrap();

    IoTaskPool::get()
        .spawn(async move {
            while let Ok(message) = socket.read() {
                match message {
                    Message::Text(text) => match serde_json::from_str::<Event>(&text) {
                        Ok(event) => {
                            EVENT_CHANNEL
                                .sender
                                .send(event)
                                .expect("error sending event to event channel");
                        }
                        Err(err) => {
                            tracing::warn!(?err, "error parsing event from websocket");
                            break;
                        }
                    },
                    other => {
                        tracing::warn!(?other, "unsupported message type received");
                        break;
                    }
                }
            }
        })
        .detach();
}
