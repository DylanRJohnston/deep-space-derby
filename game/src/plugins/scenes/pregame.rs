use bevy::prelude::*;
use shared::models::projections;

use crate::plugins::{
    event_stream::GameEvents,
    monster::{DespawnAllMonsters, MonsterBehaviour, SpawnMonster},
};

use super::{SceneMetadata, SceneState};

pub struct PreGamePlugin;

impl Plugin for PreGamePlugin {
    fn build(&self, app: &mut App) {
        app.enable_state_scoped_entities::<SceneState>()
            .register_type::<PreGameSpawnPoint>()
            .register_type::<PreGameCamera>()
            .add_systems(OnEnter(SceneState::PreGame), (spawn_monsters).chain())
            // .add_systems(
            //     Update,
            //     update_ui_position.run_if(in_state(SceneState::PreGame)),
            // )
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
    let monsters = projections::monsters(&game_events, seed);

    spawn_points
        .into_iter()
        .for_each(|(spawn_point, transform)| {
            let monster = monsters
                .get(spawn_point.id - 1)
                .ok_or_else(|| "failed to find race point for monster".to_string())
                .copied()
                .unwrap();

            commands.trigger(SpawnMonster {
                transform: *transform,
                monster,
                behaviour: MonsterBehaviour::Idle,
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
