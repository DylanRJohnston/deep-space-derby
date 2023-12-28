use std::f32::consts::PI;

use animation_link::AnimationLinkPlugin;
use asset_loader::AssetLoaderPlugin;
use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use main_menu::MainMenuPlugin;
use monster::MonsterPlugin;

mod animation_link;
mod asset_loader;
mod main_menu;
mod monster;
mod pallet;

fn main() {
    App::new()
        .add_state::<AppState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(AnimationLinkPlugin)
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(MonsterPlugin)
        .add_plugins(MainMenuPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.70,
        })
        .add_systems(Startup, setup)
        .run();
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
enum AppState {
    #[default]
    MainMenu,
    Playing,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(5.0, 10.0, 15.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(500000.0).into()),
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
