use bevy::{gltf::Gltf, pbr::CascadeShadowConfigBuilder, prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;
use bevy_registry_export::*;
use simulation::start;

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
            .add_systems(Update, orbit);
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
pub struct SpawnPoint {
    pub id: u32,
}

fn setup(
    mut commands: Commands,
    game_assets: Res<Scene>,
    models: Res<Assets<Gltf>>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(5.0, 9.0, 8.0)
    //         .looking_at(Vec3::new(0.0, 0.0, -5.0), Vec3::Y),
    //     ..default()
    // });

    ambient_light.brightness = 0.1;

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -2.5, 1.1, 0.0)),
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
