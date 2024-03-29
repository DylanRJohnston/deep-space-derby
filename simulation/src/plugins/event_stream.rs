use bevy::prelude::*;

use wasm_bindgen::prelude::*;

use lazy_static::lazy_static;

use super::scenes::SceneState;

pub struct EventStreamPlugin;

impl Plugin for EventStreamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Lobby), register_event_stream)
            .add_systems(
                Update,
                read_event_stream.run_if(not(in_state(SceneState::Loading))),
            );
    }
}

lazy_static! {
    static ref EVENT_CHANNEL: (
        crossbeam_channel::Sender<()>,
        crossbeam_channel::Receiver<()>
    ) = crossbeam_channel::unbounded::<()>();
}

#[derive(Resource)]
pub struct Channel(crossbeam_channel::Receiver<()>);

fn register_event_stream(mut commands: Commands) {
    lazy_static::initialize(&EVENT_CHANNEL);
    commands.insert_resource(Channel(EVENT_CHANNEL.1.clone()));
}

fn read_event_stream(mut next_state: ResMut<NextState<SceneState>>, channel: Res<Channel>) {
    while channel.0.try_recv().is_ok() {
        println!("Received event on channel, transitioning to race");
        next_state.set(SceneState::PreGame)
    }
}

#[wasm_bindgen(js_name = "sendGameEvent")]
pub fn send_game_event() -> Result<(), JsError> {
    EVENT_CHANNEL.0.send(()).map_err(Into::into)
}

