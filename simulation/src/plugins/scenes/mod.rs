use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping, Skybox},
    gltf::Gltf,
    pbr::{CascadeShadowConfig, CascadeShadowConfigBuilder},
    prelude::*,
    render::camera::Exposure,
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
    Connecting,
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

    #[asset(key = "envmap_diffuse")]
    envmap_diffuse: Handle<Image>,

    #[asset(key = "envmap_specular")]
    envmap_specular: Handle<Image>,
}

fn scene_setup(
    mut commands: Commands,
    game_assets: Res<Scene>,
    models: Res<Assets<Gltf>>,
    mut next_state: ResMut<NextState<SceneState>>,
    cameras: Query<Entity, With<Camera>>,
) {
    commands.spawn(SceneBundle {
        scene: models
            .get(game_assets.world.id())
            .expect("main level should have been loaded")
            .scenes[0]
            .clone(),
        ..default()
    });

    for camera in &cameras {
        commands.entity(camera).despawn_recursive();
    }

    next_state.set(SceneState::Connecting);
}

fn setup_skybox(
    mut commands: Commands,
    game_assets: Res<Scene>,
    mut camera: Query<(Entity, &mut Camera), Added<Camera>>,
    mut shadows: Query<&mut CascadeShadowConfig>,
    mut exposure: Query<&mut Exposure>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok((id, mut camera)) = camera.get_single_mut() {
        let mut bloom = BloomSettings::OLD_SCHOOL.clone();

        bloom.intensity = 0.1;
        bloom.prefilter_settings.threshold = 1.5;

        commands.entity(id).insert((
            Skybox {
                image: game_assets.skybox.clone(),
                brightness: 1000.0,
            },
            EnvironmentMapLight {
                diffuse_map: game_assets.envmap_diffuse.clone(),
                specular_map: game_assets.envmap_specular.clone(),
                intensity: 1000.0,
            },
            bloom,
            Tonemapping::AcesFitted,
            // Tonemapping::BlenderFilmic,
        ));

        camera.hdr = true;
    }

    if let Ok(mut shadow_config) = shadows.get_single_mut() {
        *shadow_config = CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 0.1,
            maximum_distance: 1000.0,
            first_cascade_far_bound: 10.0,
            ..default()
        }
        .build()
    }

    if let Ok(mut exposure) = exposure.get_single_mut() {
        exposure.ev100 = 9.0;
    }

    for (_, material) in materials.iter_mut() {
        if material.emissive_texture.is_some() {
            material.emissive = Color::rgb_linear(2000.0, 2000.0, 2000.0);
        }
    }
}
