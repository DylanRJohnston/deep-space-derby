use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping, Skybox},
    gltf::Gltf,
    pbr::{CascadeShadowConfig, CascadeShadowConfigBuilder},
    prelude::*,
    render::camera::Exposure,
    utils::{tracing, HashMap},
};
use bevy_asset_loader::prelude::*;
use results::ResultsPlugin;

use self::{lobby::LobbyPlugin, pregame::PreGamePlugin, race::RacePlugin};

use super::event_stream::GameEvents;
use shared::models::events::Event as GameEvent;

pub mod lobby;
pub mod pregame;
pub mod race;
pub mod results;

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

#[derive(Debug, Component)]
pub struct SceneMetadata(pub serde_json::Map<String, serde_json::Value>);

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut App) {
        app
            // Don't format me bro
            .add_plugins(LobbyPlugin)
            .add_plugins(RacePlugin)
            .add_plugins(PreGamePlugin)
            .add_plugins(ResultsPlugin)
            .register_type::<GltfExtras>()
            .init_state::<SceneState>()
            .add_loading_state(
                LoadingState::new(SceneState::Loading)
                    .continue_to_state(SceneState::Spawning)
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>("all.assets.ron")
                    .load_collection::<GameAssets>(),
            )
            .add_systems(Update, spawned.run_if(in_state(SceneState::Spawning)))
            .add_systems(OnEnter(SceneState::Spawning), scene_setup)
            .add_systems(OnEnter(SceneState::Lobby), setup_skybox)
            .add_systems(Update, deserialize_gltf_extras)
            .add_systems(
                Update,
                scene_manager.run_if(|state: Res<State<SceneState>>| {
                    !matches!(state.get(), SceneState::Loading | SceneState::Spawning)
                }),
            );
    }
}

fn spawned(
    scene_metadata: Query<&SceneMetadata>,
    mut scene_state: ResMut<NextState<SceneState>>,
    mut spawned: Local<bool>,
) {
    if *spawned {
        return;
    }

    if scene_metadata.iter().next().is_none() {
        return;
    }

    *spawned = true;
    scene_state.set(SceneState::Lobby);
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(key = "world")]
    pub world: Handle<Gltf>,

    #[asset(key = "models", collection(typed, mapped))]
    #[allow(dead_code)]
    pub models: HashMap<String, Handle<Gltf>>,
    // #[asset(path = "materials", collection(typed))]
    // materials: Vec<Handle<Gltf>>,
    #[asset(key = "skybox")]
    pub skybox: Handle<Image>,

    #[asset(key = "envmap_diffuse")]
    pub envmap_diffuse: Handle<Image>,

    #[asset(key = "envmap_specular")]
    pub envmap_specular: Handle<Image>,
}

fn scene_setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    models: Res<Assets<Gltf>>,
    cameras: Query<Entity, With<Camera>>,
) {
    commands.spawn((
        Name::from("Scene"),
        SceneBundle {
            scene: models
                .get(game_assets.world.id())
                .expect("main level should have been loaded")
                .scenes[0]
                .clone(),
            ..default()
        },
    ));

    for camera in &cameras {
        commands.entity(camera).despawn_recursive();
    }
}

fn setup_skybox(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut camera: Query<(Entity, &mut Camera), Added<Camera>>,
    mut shadows: Query<&mut CascadeShadowConfig>,
    mut sun: Query<&mut DirectionalLight>,
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
                brightness: 2000.0,
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

    if let Ok(mut sun) = sun.get_single_mut() {
        sun.shadows_enabled = true;
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
            material.emissive = Color::linear_rgb(2.0, 2.0, 2.0).into();
        }
    }
}

fn deserialize_gltf_extras(
    query: Query<(Entity, &Name, &GltfExtras), Added<GltfExtras>>,
    mut commands: Commands,
) {
    query.into_iter().for_each(|(entity, name, extras)| {
        match serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&extras.value) {
            Ok(metadata) => {
                commands.entity(entity).insert(SceneMetadata(metadata));
            }
            Err(_) => {
                tracing::warn!(
                    ?name, extras = ?extras.value,
                    "warning failed to deserialise gtlf metadata",
                );
            }
        }
    });
}

fn scene_manager(events: Res<GameEvents>, mut next_state: ResMut<NextState<SceneState>>) {
    if !events.is_changed() {
        return;
    }

    let state = events
        .0
        .iter()
        .rev()
        .find_map(|event| {
            Some(match event {
                GameEvent::GameCreated { .. } => SceneState::Lobby,
                GameEvent::RoundStarted { .. } => SceneState::PreGame,
                GameEvent::RaceStarted { .. } => SceneState::Race,
                GameEvent::RaceFinished { .. } => SceneState::Results,
                GameEvent::GameFinished => SceneState::Lobby,
                _ => None?,
            })
        })
        .unwrap_or(SceneState::Lobby);

    tracing::info!(?state, "setting scene state");

    next_state.set(state);
}
