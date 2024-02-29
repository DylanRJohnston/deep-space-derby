use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use iyes_progress::{ProgressCounter, ProgressPlugin, TrackedProgressSet};

use crate::plugins::{
    event_stream::EventStreamPlugin, monster::MonsterPlugin, planets::PlanetsPlugin,
    scenes::SceneState, scenes::ScenesPlugin, spectators::SpectatorPlugin,
};

pub fn start(f: impl FnOnce(&mut App)) {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(bevy_gltf_blueprints::BlueprintsPlugin {
        library_folder: "library".into(),
        material_library: false,
        legacy_mode: false,
        ..Default::default()
    })
    .add_plugins(ScenesPlugin)
    .add_plugins(EventStreamPlugin)
    .add_plugins(TweeningPlugin)
    .add_plugins(SpectatorPlugin)
    .add_plugins(PlanetsPlugin)
    .add_plugins(
        ProgressPlugin::new(SceneState::Loading)
            .continue_to(SceneState::Lobby)
            .track_assets(),
    )
    .add_systems(
        Update,
        ui_progress_bar
            .after(TrackedProgressSet)
            .run_if(in_state(SceneState::Loading)),
    )
    .add_plugins(MonsterPlugin)
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.075,
    });

    f(&mut app);

    app.run();
}

fn ui_progress_bar(counter: Res<ProgressCounter>) {
    let progress = counter.progress();

    println!("{}", Into::<f32>::into(progress));
}

