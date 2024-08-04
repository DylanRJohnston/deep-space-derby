use std::sync::LazyLock;

use bevy::{prelude::*, utils::tracing};

use crossbeam_channel::{Receiver, Sender};
use im::Vector;
use shared::models::events::Event;
use wasm_bindgen::prelude::*;

use super::scenes::SceneState;

pub struct EventStreamPlugin;

impl Plugin for EventStreamPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameEvents(Vector::new()))
            .add_systems(OnEnter(SceneState::Loading), register_event_stream)
            .add_systems(
                Update,
                read_event_stream.run_if(|state: Res<State<SceneState>>| {
                    !matches!(state.get(), SceneState::Loading | SceneState::Spawning)
                }),
            )
            .add_systems(Update, transition_debug);
    }
}

struct EventChannel {
    sender: Sender<Event>,
    receiver: Receiver<Event>,
}

static EVENT_CHANNEL: LazyLock<EventChannel> = LazyLock::new(|| {
    let (sender, receiver) = crossbeam_channel::unbounded::<Event>();

    EventChannel { sender, receiver }
});

#[derive(Resource)]
pub struct Channel(Receiver<Event>);

fn register_event_stream(mut commands: Commands) {
    commands.insert_resource(Channel(EVENT_CHANNEL.receiver.clone()));
}

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
    channel: Res<Channel>,
) {
    while let Ok(event) = channel.0.try_recv() {
        tracing::info!(?event, "Pushing event into bevy");
        events.as_mut().0.push_back(event);
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

#[wasm_bindgen(js_name = "sendGameEvent")]
pub fn send_game_event(event: Event) -> Result<(), JsError> {
    EVENT_CHANNEL.sender.send(event).map_err(Into::into)
}
