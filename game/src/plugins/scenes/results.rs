use bevy::{prelude::*, utils::tracing};
use shared::models::projections;

use crate::plugins::{
    event_stream::GameEvents,
    monster::{DespawnAllMonsters, MonsterBehaviour, SpawnMonster},
};

use super::{SceneMetadata, SceneState};

pub struct ResultsPlugin;

impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .enable_state_scoped_entities::<SceneState>()
            .add_systems(Startup, init_podium_mesh)
            .add_systems(Update, spawn_podium_on_scene_load)
            .add_systems(OnEnter(SceneState::Results), spawn_monsters)
            .add_systems(OnEnter(SceneState::Results), init_results)
            .add_systems(
                OnEnter(SceneState::Results),
                |mut query: Query<&mut Visibility, With<SpotLight>>| {
                    for mut visibility in query.iter_mut() {
                        *visibility = Visibility::Inherited;
                    }
                },
            );
    }
}

#[derive(Debug, Resource)]
struct PodiumAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

fn init_podium_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(Cylinder::new(0.4, 1.0)));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        ..default()
    });

    commands.insert_resource(PodiumAssets { mesh, material });
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct Podium {
    pub id: usize,
}

#[derive(Debug, Default, Reflect, Component)]
#[reflect(Component)]
pub struct PodiumCamera;

fn spawn_podium_on_scene_load(
    query: Query<(Entity, &SceneMetadata), Added<SceneMetadata>>,
    mut commands: Commands,
) {
    for (entity, metadata) in &query {
        if let Some(value) = metadata.0.get("Podium") {
            match value {
                serde_json::Value::Number(n) if n.is_u64() => {
                    commands.entity(entity).insert(Podium {
                        id: n.as_u64().unwrap() as usize,
                    });
                }
                other => panic!("Podium must be a number, got {:?}", other),
            }
        }

        if metadata.0.get("PodiumCamera").is_some() {
            commands.entity(entity).insert(PodiumCamera);
        }
    }
}

const PODIUM_OFFSETS: [f32; 3] = [0.75, 0.4, 0.25];

fn spawn_monsters(
    spawn_points: Query<(&Podium, &Transform)>,
    game_events: Res<GameEvents>,
    podium_assets: Res<PodiumAssets>,
    mut commands: Commands,
) {
    commands.trigger(DespawnAllMonsters);

    let seed = projections::race_seed(&game_events);
    let monsters = projections::monsters(&game_events, seed);
    let results = projections::results(&game_events).unwrap();

    spawn_points
        .into_iter()
        .for_each(|(spawn_point, transform)| {
            let uuid = match spawn_point.id {
                1 => results.first,
                2 => results.second,
                3 => results.third,
                id => {
                    tracing::warn!(?id, "spawn point with unrecognised id encountered");
                    return;
                }
            };

            let Some(monster) = monsters
                .iter()
                .find(|monster| monster.uuid == uuid)
                .copied()
            else {
                tracing::error!(?uuid, "couldn't find monster in race results");
                return;
            };

            let mut monster_transform = *transform;
            monster_transform.translation += Vec3::Y * PODIUM_OFFSETS[spawn_point.id - 1];

            commands.trigger(SpawnMonster {
                transform: monster_transform,
                monster,
                behaviour: if spawn_point.id == 1 {
                    MonsterBehaviour::Dancing
                } else {
                    MonsterBehaviour::Idle
                },
                id: spawn_point.id,
            });

            let mut block_transform = *transform;
            block_transform.scale = Vec3::new(1.0, PODIUM_OFFSETS[spawn_point.id - 1], 1.0);
            block_transform.translation.y += PODIUM_OFFSETS[spawn_point.id - 1] / 2.;

            commands.spawn((
                Name::from(format!("Podium {index}", index = spawn_point.id)),
                StateScoped(SceneState::Results),
                PbrBundle {
                    transform: block_transform,
                    mesh: podium_assets.mesh.clone(),
                    material: podium_assets.material.clone(),
                    ..default()
                },
            ));
        });
}

fn init_results(
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera>>,
    position: Query<&Transform, (With<PodiumCamera>, Without<Camera>)>,
) {
    let position = position.get_single().unwrap();
    let (mut camera, mut projection) = camera.get_single_mut().unwrap();

    camera.translation = position.translation;
    // Don't know why the rotation coming from blender is fucked up
    camera.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2) * position.rotation;

    let Projection::Perspective(projection) = projection.as_mut() else {
        return;
    };

    // camera.translation = camera.translation + camera.back() * 3.0;

    projection.fov = 0.4;
}
