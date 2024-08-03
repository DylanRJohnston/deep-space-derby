use bevy::prelude::*;
use shared::models::projections;

use crate::plugins::{
    event_stream::GameEvents,
    monster::{Monster, MonsterBundle},
};

use super::{pregame::PreGameSpawnPoint, GameAssets, SceneState};

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
            .add_systems(Update, spawn_racers)
            .add_systems(Update, orbit_camera.run_if(in_state(SceneState::Lobby)));
    }
}

pub fn init_camera(mut query: Query<&mut Transform, Added<Camera>>) {
    if let Ok(mut transform) = query.get_single_mut() {
        println!("Setting initial camera position");
        transform.translation = Vec3::new(10.0, 10.0, 10.0);
        *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);

        // projection.fov = 120.0;
    }
}

pub fn orbit_camera(mut query: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    if let Ok(mut transform) = query.get_single_mut() {
        let rot = Quat::from_axis_angle(Vec3::Y, time.elapsed_seconds() / 2.0);

        transform.translation =
            ((rot * Vec3::new(1.0, 0.5, 1.0)) + Vec3::new(1.0, 0.0, 0.0)) * 15.0;
        transform.look_at(Vec3::new(1.0, 3.0, 0.0), Vec3::Y);
    }
}

pub fn spawn_racers(
    events: Res<GameEvents>,
    mut commands: Commands,
    spawn_points: Query<(&PreGameSpawnPoint, &Transform), Added<PreGameSpawnPoint>>,
    gltfs: Res<Assets<Gltf>>,
    game_assets: Option<Res<GameAssets>>,
) {
    let monsters = projections::monsters(projections::race_seed(&events));

    spawn_points
        .into_iter()
        .for_each(|(spawn_point, transform)| {
            let monster = monsters
                .get((spawn_point.id - 1) as usize)
                .ok_or_else(|| format!("failed to find spawn point for monster: {spawn_point:?}"))
                .unwrap();

            let handle = game_assets
                .as_ref()
                .ok_or("game assets haven't loaded yet")
                .unwrap()
                .models
                .get(monster.blueprint_name)
                .ok_or_else(|| {
                    format!(
                        "failed to find asset for monster: {}, available models: {:?}",
                        monster.blueprint_name,
                        game_assets.as_ref().unwrap().models.keys()
                    )
                })
                .unwrap();

            let scene = gltfs
                .get(handle)
                .ok_or_else(|| {
                    format!(
                        "failed to retrieve asset for monster: {}",
                        monster.blueprint_name
                    )
                })
                .unwrap();

            let mut transform = *transform;
            transform.scale = Vec3::splat(0.25);

            commands.spawn((
                Name::from(monster.name),
                MonsterBundle {
                    monster: Monster::default(),
                    speed: monster.speed.into(),
                    ..default()
                },
                handle.clone(),
                SceneBundle {
                    scene: scene.scenes[0].clone(),
                    transform: transform,
                    ..default()
                },
            ));
        });
}
