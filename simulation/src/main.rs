use std::f32::consts::PI;

use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use iyes_progress::{ProgressCounter, ProgressPlugin, TrackedProgressSet};
use plugins::animation_link::AnimationLinkPlugin;
use plugins::asset_loader::load_assets;
use plugins::event_stream::EventStreamPlugin;
use plugins::fetch_data::FetchDataPlugin;
use plugins::menus::MenuPlugin;
use plugins::monster::MonsterPlugin;

mod plugins;

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
enum AppState {
    #[default]
    Splash,
    MainMenu,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EventStreamPlugin)
        .add_state::<AppState>()
        .add_plugins(
            ProgressPlugin::new(AppState::Splash)
                .continue_to(AppState::MainMenu)
                .track_assets(),
        )
        .add_systems(OnEnter(AppState::Splash), load_assets)
        .add_systems(
            Update,
            ui_progress_bar
                .after(TrackedProgressSet)
                .run_if(in_state(AppState::Splash)),
        )
        .add_plugins(AnimationLinkPlugin)
        .add_plugins(MonsterPlugin)
        // .add_plugins(MenuPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.70,
        })
        .add_plugins(FetchDataPlugin)
        .add_systems(Startup, setup)
        // .add_plugins(WorldInspectorPlugin::new())
        .run();
}

fn ui_progress_bar(counter: Res<ProgressCounter>) {
    let progress = counter.progress();

    println!("{}", Into::<f32>::into(progress));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(5.0, 9.0, 8.0)
            .looking_at(Vec3::new(0.0, 0.0, -5.0), Vec3::Y),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(10000.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 20.0,
            maximum_distance: 40.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

