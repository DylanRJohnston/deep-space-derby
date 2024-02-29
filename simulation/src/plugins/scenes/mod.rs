use bevy::{gltf::Gltf, pbr::CascadeShadowConfigBuilder, prelude::*, utils::HashMap};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
};

use self::{lobby::LobbyPlugin, pregame::PreGamePlugin, race::RacePlugin};

pub mod lobby;
pub mod pregame;
pub mod race;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum SceneState {
    #[default]
    Loading,
    Lobby,
    PreGame,
    Race,
    Results,
}

pub struct ScenesPlugin;

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut App) {
        app
            // Don't format me bro
            .add_plugins(LobbyPlugin)
            .add_plugins(RacePlugin)
            .add_plugins(PreGamePlugin)
            .add_state::<SceneState>()
            .add_loading_state(
                LoadingState::new(SceneState::Loading)
                    .continue_to_state(SceneState::Lobby)
                    .load_collection::<Scene>(),
            )
            .add_systems(OnEnter(SceneState::Lobby), scene_setup);
    }
}

#[derive(AssetCollection, Resource)]
struct Scene {
    #[asset(path = "Scene.glb")]
    world: Handle<Gltf>,

    #[asset(path = "library", collection(typed, mapped))]
    #[allow(dead_code)]
    models: HashMap<String, Handle<Gltf>>,
    // #[asset(path = "materials", collection(typed))]
    // materials: Vec<Handle<Gltf>>,
}

fn scene_setup(mut commands: Commands, game_assets: Res<Scene>, models: Res<Assets<Gltf>>) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.6, 0.9, 0.0)),
        directional_light: DirectionalLight {
            illuminance: 75000.0,
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            maximum_distance: 40.0,
            ..default()
        }
        .into(),
        ..default()
    });

    commands.spawn(SceneBundle {
        scene: models
            .get(game_assets.world.id())
            .expect("main level should have been loaded")
            .scenes[0]
            .clone(),
        ..default()
    });
}

