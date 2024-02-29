use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use iyes_progress::{ProgressCounter, ProgressPlugin, TrackedProgressSet};

use crate::plugins::{
    asset_loader::load_assets, event_stream::EventStreamPlugin, fetch_data::FetchDataPlugin,
    monster::MonsterPlugin, spectators::SpectatorPlugin,
};

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AppState {
    #[default]
    Splash,
    Lobby,
    PreGame,
    Race,
    Results,
}

pub fn start(f: impl FnOnce(&mut App)) {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(EventStreamPlugin)
    .add_state::<AppState>()
    .add_plugins(TweeningPlugin)
    .add_plugins(SpectatorPlugin)
    .add_plugins(
        ProgressPlugin::new(AppState::Splash)
            .continue_to(AppState::Lobby)
            .track_assets(),
    )
    .add_systems(OnEnter(AppState::Splash), load_assets)
    .add_systems(
        Update,
        ui_progress_bar
            .after(TrackedProgressSet)
            .run_if(in_state(AppState::Splash)),
    )
    .add_plugins(MonsterPlugin)
    // .add_plugins(MenuPlugin)
    .insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.70,
    })
    .add_plugins(FetchDataPlugin);

    f(&mut app);

    app.run();
}

fn ui_progress_bar(counter: Res<ProgressCounter>) {
    let progress = counter.progress();

    println!("{}", Into::<f32>::into(progress));
}
