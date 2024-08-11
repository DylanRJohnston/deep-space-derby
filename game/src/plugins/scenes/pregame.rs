use bevy::{prelude::*, utils::tracing};
use shared::models::projections;

use crate::plugins::{
    event_stream::GameEvents,
    monster::{DespawnAllMonsters, MonsterID, MonsterRef, SpawnMonster},
};

use super::{SceneMetadata, SceneState};

pub struct PreGamePlugin;

impl Plugin for PreGamePlugin {
    fn build(&self, app: &mut App) {
        app.enable_state_scoped_entities::<SceneState>()
            .register_type::<PreGameSpawnPoint>()
            .register_type::<PreGameCamera>()
            .add_systems(
                OnEnter(SceneState::PreGame),
                (spawn_monsters, spawn_ui).chain(),
            )
            .add_systems(
                Update,
                update_ui_position.run_if(in_state(SceneState::PreGame)),
            )
            .add_systems(
                OnEnter(SceneState::PreGame),
                init_pregame.after(spawn_pregame_spawn_point_on_scene_load),
            )
            .add_systems(Update, spawn_pregame_spawn_point_on_scene_load)
            .add_systems(
                OnEnter(SceneState::PreGame),
                |mut query: Query<&mut Visibility, With<SpotLight>>| {
                    for mut visibility in query.iter_mut() {
                        *visibility = Visibility::Inherited;
                    }
                },
            );
    }
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct PreGameSpawnPoint {
    pub id: usize,
}

#[derive(Debug, Default, Reflect, Component)]
#[reflect(Component)]
pub struct PreGameCamera;

pub fn spawn_pregame_spawn_point_on_scene_load(
    query: Query<(Entity, &SceneMetadata), Added<SceneMetadata>>,
    mut commands: Commands,
) {
    for (entity, metadata) in &query {
        if let Some(value) = metadata.0.get("PreGameSpawnPoint") {
            tracing::info!("Spawning pregame spawn points");
            match value {
                serde_json::Value::Number(n) if n.is_u64() => {
                    commands.entity(entity).insert(PreGameSpawnPoint {
                        id: n.as_u64().unwrap() as usize,
                    });
                }
                other => panic!("PreGameSpawnPoint must be a number, got {:?}", other),
            }
        }

        if metadata.0.get("PreGameCamera").is_some() {
            commands.entity(entity).insert(PreGameCamera);
        }
    }
}

fn spawn_monsters(
    spawn_points: Query<(&PreGameSpawnPoint, &Transform)>,
    game_events: Res<GameEvents>,
    mut commands: Commands,
) {
    commands.trigger(DespawnAllMonsters);

    let seed = projections::race_seed(&game_events);
    let monsters = projections::monsters(seed);

    spawn_points
        .into_iter()
        .for_each(|(spawn_point, transform)| {
            let monster = monsters
                .get(spawn_point.id - 1)
                .ok_or_else(|| "failed to find race point for monster".to_string())
                .unwrap();

            commands.trigger(SpawnMonster {
                transform: *transform,
                monster,
                id: spawn_point.id,
            })
        });
}

fn init_pregame(
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera>>,
    position: Query<&Transform, (With<PreGameCamera>, Without<Camera>)>,
) {
    let position = position.get_single().unwrap();
    let (mut camera, mut projection) = camera.get_single_mut().unwrap();

    camera.translation = position.translation;
    // Don't know why the rotation coming from blender is fucked up
    camera.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2) * position.rotation;

    let Projection::Perspective(projection) = projection.as_mut() else {
        return;
    };

    projection.fov = 0.4;
}

#[derive(Debug, Component)]
struct StatID(Entity);

fn spawn_ui(
    mut commands: Commands,
    camera: Query<(&Camera, &GlobalTransform)>,
    monsters: Query<(Entity, &MonsterRef, &GlobalTransform), With<MonsterID>>,
) {
    let (camera, camera_transform) = camera.get_single().unwrap();

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        })
        .with_children(|container| {
            for (id, details, monster) in &monsters {
                let Some(position) =
                    camera.world_to_viewport(camera_transform, monster.translation())
                else {
                    return;
                };

                tracing::info!(?position);

                container
                    .spawn((
                        StatID(id),
                        StateScoped(SceneState::PreGame),
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                top: Val::Px(position.y),
                                left: Val::Px(position.x),
                                ..default()
                            },
                            background_color: Color::WHITE.into(),
                            ..default()
                        },
                    ))
                    .with_children(|container| {
                        container.spawn(TextBundle {
                            text: Text::from_section(
                                details.name,
                                TextStyle {
                                    font_size: 20.0,
                                    color: Color::BLACK,
                                    ..default()
                                },
                            ),
                            ..default()
                        });
                    });
            }
        });
}

fn update_ui_position(
    camera: Query<(&Camera, &GlobalTransform)>,
    monsters: Query<&GlobalTransform>,
    mut ui_nodes: Query<(&mut Style, &StatID)>,
) {
    let (camera, camera_transform) = camera.get_single().unwrap();

    for (mut style, StatID(entity)) in &mut ui_nodes {
        let monster = monsters.get(*entity).unwrap();

        let Some(position) = camera.world_to_viewport(camera_transform, monster.translation())
        else {
            continue;
        };

        style.top = Val::Px(position.y);
        style.left = Val::Px(position.x);
    }
}
