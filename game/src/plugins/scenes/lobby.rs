use bevy::prelude::*;
use shared::models::projections;

use crate::plugins::{
    event_stream::GameEvents,
    monster::{MonsterBehaviour, SpawnMonster},
};

use super::{pregame::PreGameSpawnPoint, SceneState};

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Lobby), init_camera)
            .add_systems(
                OnEnter(SceneState::Lobby),
                |mut query: Query<&mut Visibility, With<SpotLight>>| {
                    for mut visibility in query.iter_mut() {
                        *visibility = Visibility::Hidden;
                    }
                },
            )
            .add_systems(OnEnter(SceneState::Lobby), spawn_racers)
            .add_systems(Update, orbit_camera.run_if(in_state(SceneState::Lobby)));
    }
}

pub fn init_camera(mut query: Query<&mut Transform, Added<Camera>>) {
    if let Ok(mut transform) = query.get_single_mut() {
        transform.translation = Vec3::new(10.0, 10.0, 10.0);
        *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);

        // projection.fov = 120.0;
    }
}

pub fn orbit_camera(mut query: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    if let Ok(mut transform) = query.get_single_mut() {
        let rot = Quat::from_axis_angle(Vec3::Y, time.elapsed_secs() / 2.0);

        transform.translation =
            ((rot * Vec3::new(1.0, 0.5, 1.0)) + Vec3::new(1.0, 0.0, 0.0)) * 15.0;
        transform.look_at(Vec3::new(1.0, 3.0, 0.0), Vec3::Y);
    }
}

pub fn spawn_racers(
    events: Res<GameEvents>,
    mut commands: Commands,
    spawn_points: Query<(&PreGameSpawnPoint, &Transform), Added<PreGameSpawnPoint>>,
) {
    let monsters = projections::monsters(&events, projections::race_seed_for_round(&events, 1));

    spawn_points
        .into_iter()
        .for_each(move |(spawn_point, transform)| {
            let monster = monsters
                .get(spawn_point.id - 1)
                .ok_or_else(|| format!("failed to find spawn point for monster: {spawn_point:?}"))
                .copied()
                .unwrap();

            commands.trigger(SpawnMonster {
                id: spawn_point.id,
                transform: *transform,
                monster,
                behaviour: MonsterBehaviour::Dancing,
                ..default()
            })
        });
}
