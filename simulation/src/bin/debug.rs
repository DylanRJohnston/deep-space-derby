use std::ops::{Add, DerefMut};

use bevy::prelude::ResMut;
use bevy::render::camera::Exposure;
use bevy::state::state::OnEnter;
use bevy::{app::Startup, ecs::system::Commands};
use shared::models::events::Event;
use shared::models::game_id::GameID;
use simulation::plugins::event_stream::Events;
use simulation::plugins::scenes::SceneState;
use simulation::{plugins::event_stream::Seed, start};

#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(feature = "debug")]
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

fn main() {
    start(|app| {
        #[cfg(feature = "debug")]
        app.add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(WorldInspectorPlugin::default());

        app.register_type::<Exposure>()
            .add_systems(Startup, |mut commands: Commands| {
                commands.insert_resource(Seed(2))
            });

        app.add_systems(OnEnter(SceneState::Lobby), |mut events: ResMut<Events>| {
            events.deref_mut().0.push_back(Event::GameCreated {
                game_id: GameID::try_from("ABCDEF").unwrap(),
            });
        });
    });
}
