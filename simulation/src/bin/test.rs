use bevy::{gltf::Gltf, pbr::CascadeShadowConfigBuilder, prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;
use bevy_gltf_blueprints::{BluePrintBundle, BlueprintName, GltfBlueprintsSet};
use bevy_registry_export::*;
use simulation::{
    plugins::monster::{MonsterBundle, Speed, Stats},
    start,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum SceneState {
    #[default]
    Loading,
    Loaded,
}

fn main() {
    start(|app| {
        app.register_type::<RotateSpeed>()
            .register_type::<OrbitPoint>()
            .register_type::<RaceSpawnPoint>()
            .register_type::<PreGameSpawnPoint>()
            .register_type::<PreGameCamera>()
            .register_type::<RaceStartCamera>()
            .add_plugins(bevy_gltf_blueprints::BlueprintsPlugin {
                library_folder: "library".into(),
                material_library: false,
                legacy_mode: false,
                ..Default::default()
            })
            .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
            .add_state::<SceneState>()
            .add_loading_state(
                LoadingState::new(SceneState::Loading)
                    .continue_to_state(SceneState::Loaded)
                    .load_collection::<Scene>(),
            )
            .add_plugins(ExportRegistryPlugin::default())
            .add_systems(OnEnter(SceneState::Loaded), setup)
            .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
            .add_systems(Update, rotate)
            .add_systems(Update, orbit)
            .add_systems(Update, find_spawn.in_set(GltfBlueprintsSet::AfterSpawn));
    });
}

#[derive(AssetCollection, Resource)]
struct Scene {
    #[asset(path = "Scene.glb")]
    world: Handle<Gltf>,

    #[asset(path = "library", collection(typed, mapped))]
    models: HashMap<String, Handle<Gltf>>,
    // #[asset(path = "materials", collection(typed))]
    // materials: Vec<Handle<Gltf>>,
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct RotateSpeed {
    pub speed: f32,
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct OrbitPoint {
    pub velocity: f32,
    pub point: Vec3,
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct RaceSpawnPoint {
    pub id: u32,
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct PreGameSpawnPoint {
    pub id: u32,
}

fn find_spawn(
    mut commands: Commands,
    query: Query<(&RaceSpawnPoint, &Transform), Added<RaceSpawnPoint>>,
) {
    for (spawn_point, transform) in &query {
        if spawn_point.id == 0 {
            continue;
        }

        println!(
            "Found race spawn point {:?} at {:?}",
            spawn_point, transform
        );

        let mut name = "Monster_Alien";

        if spawn_point.id == 2 {
            name = "Monster_Chicken";
        }

        if spawn_point.id == 3 {
            name = "Monster_Mushnub";
        }

        commands.spawn((
            BluePrintBundle {
                blueprint: BlueprintName(name.into()),
                ..default()
            },
            MonsterBundle {
                speed: Speed(1.0),
                stats: Stats { recovery_time: 2.0 },
                ..default()
            },
            *transform,
        ));
    }
}

fn setup(
    mut commands: Commands,
    game_assets: Res<Scene>,
    models: Res<Assets<Gltf>>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    ambient_light.brightness = 0.075;

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

pub fn rotate(mut query: Query<(&mut Transform, &RotateSpeed)>, time: Res<Time>) {
    for (mut transform, rotation) in &mut query {
        transform.rotate_y(time.delta_seconds() * rotation.speed);
    }
}

pub fn orbit(mut query: Query<(&mut Transform, &OrbitPoint)>, time: Res<Time>) {
    for (mut transform, orbit) in &mut query {
        transform.translate_around(
            orbit.point,
            Quat::from_rotation_y(time.delta_seconds() * orbit.velocity),
        )
    }
}

#[derive(Debug, Default, Reflect, Component)]
#[reflect(Component)]
pub struct PreGameCamera;

#[derive(Debug, Default, Reflect, Component)]
#[reflect(Component)]
pub struct RaceStartCamera;

