use bevy::{prelude::*, transform::commands, utils::tracing};
use shared::models::{
    monsters::{self, RaceResults},
    projections,
};

use crate::plugins::{
    event_stream::GameEvents,
    monster::{BehaviourTimer, DespawnAllMonsters, MonsterBehaviour, MonsterID, SpawnMonster},
};

use super::{SceneMetadata, SceneState};

pub struct RacePlugin;

impl Plugin for RacePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RaceSpawnPoint>()
            .register_type::<RaceStartCamera>()
            .add_systems(Update, spawn_race_spawn_point_on_scene_load)
            .add_systems(OnEnter(SceneState::Race), init_race)
            .add_systems(Update, run_race.run_if(in_state(SceneState::Race)))
            .observe(reset_race);

        #[cfg(feature = "debug")]
        app.add_systems(Update, debug_reset_race);
    }
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct RaceSpawnPoint {
    pub id: usize,
}

#[derive(Debug, Default, Reflect, Component)]
#[reflect(Component)]
pub struct RaceStartCamera;

pub fn spawn_race_spawn_point_on_scene_load(
    query: Query<(Entity, &SceneMetadata), Added<SceneMetadata>>,
    mut commands: Commands,
) {
    for (entity, metadata) in &query {
        if let Some(value) = metadata.0.get("RaceSpawnPoint") {
            match value {
                serde_json::Value::Number(n) if n.is_u64() => {
                    commands.entity(entity).insert(RaceSpawnPoint {
                        id: n.as_u64().unwrap() as usize,
                    });
                }
                other => panic!("RaceSpawnPoint must be a number, got {:?}", other),
            }
        }

        if metadata.0.get("RaceStartCamera").is_some() {
            commands.entity(entity).insert(RaceStartCamera);
        }
    }
}

// error[B0001]: Query<(&game::plugins::scenes::race::RaceSpawnPoint, &bevy_transform::components::transform::Transform), ()> in system game::plugins::scenes::race::init_race accesses component(s) bevy_transform::components::transform::Transform in a way that conflicts with a previous system parameter. Consider using `Without<T>` to create disjoint Queries or merging conflicting Queries into a `ParamSet`. See: https://bevyengine.org/learn/errors/#b0001

#[derive(Debug, Event)]
struct InitRace;

fn init_race(mut commands: Commands) {
    commands.trigger(InitRace);
}

#[cfg(feature = "debug")]
fn debug_reset_race(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::KeyR) {
        commands.trigger(InitRace);
    }
}

fn reset_race(
    _trigger: Trigger<InitRace>,
    position: Query<&Transform, (With<RaceStartCamera>, Without<Camera>)>,
    race_points: Query<(&RaceSpawnPoint, &Transform), Without<Camera>>,
    game_events: Res<GameEvents>,
    mut commands: Commands,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    commands.trigger(DespawnAllMonsters);

    let position = position.get_single().unwrap();
    let mut camera = camera.get_single_mut().unwrap();

    camera.translation = position.translation;
    // Don't know why the rotation coming from blender is fucked up
    camera.rotation = position.rotation * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);

    let seed = projections::race_seed(&game_events);
    let monsters = projections::monsters(seed);
    let result = monsters::race(&monsters, seed);

    commands.insert_resource(Race(result));

    race_points
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

#[derive(Debug, Component)]
pub struct RaceTimer {
    index: usize,
    timer: Timer,
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct Race(RaceResults);

const RACE_SPEED: f32 = 0.58;

impl Default for RaceTimer {
    fn default() -> Self {
        Self {
            index: 0,
            timer: Timer::from_seconds(0., TimerMode::Once),
        }
    }
}

fn run_race(
    race: Res<Race>,
    time: Res<Time>,
    mut monsters: Query<(&MonsterID, &mut BehaviourTimer, &mut RaceTimer)>,
) {
    for (id, mut behaviour_timer, mut race_timer) in &mut monsters {
        if !race_timer.timer.tick(time.delta()).just_finished() {
            continue;
        }

        let Some(round) = race.rounds.get(race_timer.index) else {
            continue;
        };

        *race_timer = RaceTimer {
            index: race_timer.index + 1,
            timer: Timer::from_seconds(
                0.0 * (rand::random::<f32>() - 0.5) + RACE_SPEED,
                TimerMode::Once,
            ),
        };

        let Some(moves) = round.get(**id - 1) else {
            tracing::warn!(?id, "no moves for monster");
            continue;
        };

        if *moves <= 0.01 {
            if behaviour_timer.next_state != MonsterBehaviour::Dancing {
                *behaviour_timer = BehaviourTimer {
                    timer: Timer::from_seconds(0., TimerMode::Once),
                    next_state: MonsterBehaviour::Dancing,
                };
            }

            continue;
        }

        *behaviour_timer = BehaviourTimer {
            timer: Timer::from_seconds(0., TimerMode::Once),
            next_state: MonsterBehaviour::Jumping(*moves),
        };
    }
}
