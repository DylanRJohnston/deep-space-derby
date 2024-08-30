use bevy::prelude::*;
use shared::models::projections::{self, Jump, RaceResults};

use crate::plugins::{
    delayed_command::DelayedCommandExt,
    event_stream::GameEvents,
    monster::{DespawnAllMonsters, MonsterBehaviour, MonsterID, MonsterInfo, SpawnMonster},
    music::PlayCountdown,
};

use super::{
    pregame::{PreGameCamera, PreGameSpawnPoint},
    RaceState, SceneMetadata,
};

pub struct RacePlugin;

impl Plugin for RacePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RaceSpawnPoint>()
            .register_type::<RaceStartCamera>()
            .add_systems(Update, spawn_race_spawn_point_on_scene_load)
            .add_systems(OnEnter(RaceState::PreRace), init_pre_race)
            .add_systems(Update, pre_race_timer.run_if(in_state(RaceState::PreRace)))
            .add_systems(OnEnter(RaceState::Race), init_race)
            .add_systems(Update, run_race.run_if(in_state(RaceState::Race)))
            .add_systems(Update, race_camera.run_if(in_state(RaceState::Race)));

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

fn init_pre_race(
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera>>,
    position: Query<&Transform, (With<PreGameCamera>, Without<Camera>)>,
    spawn_points: Query<(&PreGameSpawnPoint, &Transform), Without<Camera>>,
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

    let position = position.get_single().unwrap();
    let (mut camera, mut projection) = camera.get_single_mut().unwrap();

    camera.translation = position.translation;
    // Don't know why the rotation coming from blender is fucked up
    camera.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2) * position.rotation;

    let Projection::Perspective(projection) = projection.as_mut() else {
        return;
    };

    projection.fov = 0.4;

    let pre_race_duration = projections::pre_race_duration(&game_events);

    commands.insert_resource(PreRaceTimer(Timer::new(pre_race_duration, TimerMode::Once)));

    commands.delayed(
        f32::max(pre_race_duration.as_secs_f32() - 3.0, 0.0),
        |commands| commands.trigger(PlayCountdown),
    );
}

#[derive(Debug, Resource)]
struct PreRaceTimer(Timer);

fn pre_race_timer(
    time: Res<Time>,
    timer: Option<ResMut<PreRaceTimer>>,
    mut state: ResMut<NextState<RaceState>>,
) {
    let Some(mut timer) = timer else {
        return;
    };

    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    state.set(RaceState::Race);
}

fn init_race(
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
    let monsters = projections::monsters(&game_events, seed);
    let (results, jump) = projections::race(&monsters, seed);

    commands.insert_resource(Race((results, jump)));

    race_points
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

#[derive(Debug, Component)]
pub struct RaceTimer {
    index: usize,
    timer: Timer,
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct Race((RaceResults, Vec<Jump>));

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
    mut monsters: Query<(
        &MonsterID,
        &MonsterInfo,
        &mut MonsterBehaviour,
        &mut RaceTimer,
    )>,
) {
    for (id, monster_info, mut behaviour_timer, mut race_timer) in &mut monsters {
        if !race_timer.timer.tick(time.delta()).just_finished() {
            continue;
        }

        let Some(jump) = race
            .0
             .1
            .iter()
            .filter(|jump| jump.monster_id == (**id - 1))
            .nth(race_timer.index)
        else {
            // if race.0 .0.first == monster_info.uuid {
            *behaviour_timer = MonsterBehaviour::Dancing;
            // } else {
            // *behaviour_timer = MonsterBehaviour::Dead;
            // }

            continue;
        };

        let timer = jump.end - jump.start;

        *race_timer = RaceTimer {
            index: race_timer.index + 1,
            timer: Timer::from_seconds(f32::max(timer, 0.0), TimerMode::Once),
        };

        *behaviour_timer = MonsterBehaviour::Jumping(*jump);
    }
}

fn race_camera(
    monsters: Query<(Entity, &Transform), (With<RaceTimer>, Without<Camera>)>,
    time: Res<Time>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera>>,
    mut tracking_timer: Local<Timer>,
    mut leader_id: Local<Option<Entity>>,
) {
    let first = || {
        monsters
            .iter()
            .max_by(|(_, a), (_, b)| a.translation.x.total_cmp(&b.translation.x))
            .unwrap()
    };

    if tracking_timer.tick(time.delta()).finished() {
        let monster = first().0;

        *leader_id = Some(monster);
        *tracking_timer = Timer::from_seconds(1.0, TimerMode::Once);
    }

    let transform = match leader_id.and_then(|id| monsters.get(id).ok()) {
        Some((_, transform)) => transform,
        None => first().1,
    };

    let (mut camera, mut projection) = camera.get_single_mut().unwrap();

    let Projection::Perspective(ref mut projection) = *projection else {
        panic!("Camera is not a PerspectiveProjection");
    };

    projection.fov = projection.fov.lerp(
        0.05 + (transform.translation.x - 4.0) / 12. * 0.3,
        time.delta_seconds(),
    );

    camera.rotation = camera.rotation.lerp(
        camera.looking_at(transform.translation, Vec3::Y).rotation,
        time.delta_seconds(),
    );
}
