use std::sync::{
    self,
    mpsc::{channel, Receiver, Sender},
    LazyLock, Mutex,
};

use bevy::{
    prelude::*,
    utils::{synccell::SyncCell, tracing},
};

use im::Vector;
use shared::models::{events::Event, events::EventStream, game_code};

pub struct EventStreamPlugin;

impl Plugin for EventStreamPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameEvents(Vector::new()))
            .add_systems(Startup, init_event_stream)
            .add_systems(Update, read_event_stream);

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Startup, connect_to_server);
    }
}

struct EventChannel<T> {
    sender: Sender<T>,
    receiver: Mutex<Option<Receiver<T>>>,
}

static EVENT_CHANNEL: LazyLock<EventChannel<EventStream>> = LazyLock::new(|| {
    let (sender, receiver) = channel::<EventStream>();

    EventChannel {
        sender,
        receiver: Mutex::new(Some(receiver)),
    }
});

#[derive(Resource)]
pub struct Seed(pub u32);

#[derive(Resource)]
pub struct GameEvents(pub Vector<Event>);

#[derive(Resource, Deref, DerefMut)]
pub struct EventReceiver(SyncCell<Receiver<EventStream>>);

impl std::ops::Deref for GameEvents {
    type Target = Vector<Event>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn init_event_stream(mut commands: Commands) {
    let receiver = EVENT_CHANNEL.receiver.lock().unwrap().take().unwrap();

    commands.insert_resource(EventReceiver(SyncCell::new(receiver)));
}

fn read_event_stream(mut receiver: ResMut<EventReceiver>, mut events: ResMut<GameEvents>) {
    while let Ok(new_events) = receiver.get().try_recv() {
        match new_events {
            EventStream::Events(new_events) => events.as_mut().0 = Vector::from(new_events),
            EventStream::Event(new_event) => events.as_mut().0.push_back(new_event),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(js_name = "sendGameEvent")]
pub fn send_game_event(events: EventStream) -> Result<(), wasm_bindgen::JsError> {
    EVENT_CHANNEL.sender.send(events).map_err(Into::into)
}

#[derive(Debug, Resource, Deref)]
pub struct GameCode(pub game_code::GameCode);

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
                                .send(EventStream::Event(event))
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
