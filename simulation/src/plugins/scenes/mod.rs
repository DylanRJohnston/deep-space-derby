use bevy::{
    core_pipeline::Skybox,
    gltf::Gltf,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
    utils::HashMap,
};
use bevy_asset_loader::prelude::*;

use self::{lobby::LobbyPlugin, pregame::PreGamePlugin, race::RacePlugin};

pub mod lobby;
pub mod pregame;
pub mod race;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum SceneState {
    #[default]
    Loading,
    Spawning,
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
            .init_state::<SceneState>()
            .add_loading_state(
                LoadingState::new(SceneState::Loading)
                    .continue_to_state(SceneState::Spawning)
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>("all.assets.ron")
                    .load_collection::<Scene>(),
            )
            .add_systems(OnEnter(SceneState::Spawning), scene_setup)
            .add_systems(OnEnter(SceneState::Lobby), setup_skybox);
    }
}

#[derive(AssetCollection, Resource)]
struct Scene {
    #[asset(key = "world")]
    world: Handle<Gltf>,

    #[asset(key = "models", collection(typed, mapped))]
    #[allow(dead_code)]
    models: HashMap<String, Handle<Gltf>>,
    // #[asset(path = "materials", collection(typed))]
    // materials: Vec<Handle<Gltf>>,
    #[asset(key = "skybox")]
    skybox: Handle<Image>,
}

fn scene_setup(
    mut commands: Commands,
    game_assets: Res<Scene>,
    models: Res<Assets<Gltf>>,
    mut images: ResMut<Assets<Image>>,
    mut next_state: ResMut<NextState<SceneState>>,
    cameras: Query<Entity, With<Camera>>,
) {
    let skybox = images.get_mut(&game_assets.skybox).unwrap();
    skybox.reinterpret_stacked_2d_as_array(6);
    skybox.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });

    for camera in &cameras {
        commands.entity(camera).despawn_recursive();
    }

    commands.spawn(SceneBundle {
        scene: models
            .get(game_assets.world.id())
            .expect("main level should have been loaded")
            .scenes[0]
            .clone(),
        ..default()
    });

    next_state.set(SceneState::Lobby);
}

fn setup_skybox(
    mut commands: Commands,
    game_assets: Res<Scene>,
    mut camera: Query<Entity, Added<Camera>>,
) {
    if let Ok(camera) = camera.get_single_mut() {
        commands.entity(camera).insert(Skybox {
            image: game_assets.skybox.clone(),
            brightness: 1000.0,
        });
    }
}
