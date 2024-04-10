use bevy::prelude::*;

use shared::models::events::SceneEvent;
use wasm_bindgen::prelude::*;

use lazy_static::lazy_static;

use super::scenes::SceneState;

pub struct EventStreamPlugin;

impl Plugin for EventStreamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Loading), register_event_stream)
            .add_systems(
                Update,
                read_event_stream.run_if(not(in_state(SceneState::Loading))),
            );
    }
}

lazy_static! {
    static ref EVENT_CHANNEL: (
        crossbeam_channel::Sender<SceneEvent>,
        crossbeam_channel::Receiver<SceneEvent>
    ) = crossbeam_channel::unbounded::<SceneEvent>();
}

#[derive(Resource)]
pub struct Channel(crossbeam_channel::Receiver<SceneEvent>);

fn register_event_stream(mut commands: Commands) {
    lazy_static::initialize(&EVENT_CHANNEL);
    commands.insert_resource(Channel(EVENT_CHANNEL.1.clone()));
}

#[derive(Resource)]
pub struct Seed(pub u32);

fn read_event_stream(
    mut commands: Commands,
    mut next_state: ResMut<NextState<SceneState>>,
    channel: Res<Channel>,
) {
    loop {
        match channel.0.try_recv() {
            Ok(SceneEvent::Lobby { seed }) => {
                next_state.set(SceneState::Lobby);
                commands.insert_resource(Seed(seed))
            }
            Ok(SceneEvent::PreGame { seed }) => {
                next_state.set(SceneState::PreGame);
                commands.insert_resource(Seed(seed))
            }
            Ok(SceneEvent::Race { seed }) => {
                next_state.set(SceneState::Race);
                commands.insert_resource(Seed(seed))
            }
            Ok(SceneEvent::Results { seed }) => {
                next_state.set(SceneState::Results);
                commands.insert_resource(Seed(seed))
            }
            Err(_) => break,
        }
    }
}

#[wasm_bindgen(js_name = "sendGameEvent")]
pub fn send_game_event(event: SceneEvent) -> Result<(), JsError> {
    EVENT_CHANNEL.0.send(event).map_err(Into::into)
}
