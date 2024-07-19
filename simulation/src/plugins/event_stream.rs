use std::sync::LazyLock;

use bevy::prelude::*;

use crossbeam_channel::{Receiver, Sender};
use im::Vector;
use shared::models::events::Event;
use wasm_bindgen::prelude::*;

use super::scenes::SceneState;

pub struct EventStreamPlugin;

impl Plugin for EventStreamPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Events(Vector::new()))
            .add_systems(OnEnter(SceneState::Loading), register_event_stream)
            .add_systems(
                Update,
                read_event_stream.run_if(|state: Res<State<SceneState>>| {
                    !matches!(state.get(), SceneState::Loading | SceneState::Spawning)
                }),
            )
            .add_systems(Update, scene_manager.run_if(resource_changed::<Events>));
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
pub struct Events(pub Vector<Event>);

fn read_event_stream(
    // mut next_state: ResMut<NextState<SceneState>>,
    mut events: ResMut<Events>,
    channel: Res<Channel>,
) {
    while let Ok(event) = channel.0.try_recv() {
        events.as_mut().0.push_back(event);
    }
}

fn scene_manager(events: Res<Events>, mut next_state: ResMut<NextState<SceneState>>) {
    if !events.is_changed() {
        return;
    }

    for event in events.0.iter() {
        match event {
            Event::GameCreated { game_id } => next_state.set(SceneState::Lobby),
            Event::GameStarted => next_state.set(SceneState::Lobby),
            Event::RaceStarted => next_state.set(SceneState::Lobby),
            Event::RaceFinished(_) => next_state.set(SceneState::Lobby),
            Event::GameFinished => next_state.set(SceneState::Lobby),
            _ => {}
        }
    }
}

#[wasm_bindgen(js_name = "sendGameEvent")]
pub fn send_game_event(event: Event) -> Result<(), JsError> {
    EVENT_CHANNEL.sender.send(event).map_err(Into::into)
}
