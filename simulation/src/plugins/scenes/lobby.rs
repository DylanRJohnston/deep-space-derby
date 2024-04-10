use bevy::prelude::*;
use bevy_gltf_blueprints::{BlueprintName, GltfBlueprintsSet, SpawnHere};
use shared::models::projections;
use wasm_bindgen::JsValue;

use crate::plugins::{
    event_stream::Seed,
    monster::{self, Monster, MonsterBundle},
};

use super::{pregame::PreGameSpawnPoint, SceneState};

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Lobby), init_camera)
            .add_systems(Update, spawn_monsters.after(GltfBlueprintsSet::AfterSpawn))
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

pub fn spawn_monsters(
    mut commands: Commands,
    seed: Option<Res<Seed>>,
    spawn_points: Query<(&PreGameSpawnPoint, &Transform), Added<PreGameSpawnPoint>>,
) {
    if seed.is_none() {
        return;
    }

    let seed = seed.unwrap();
    let monsters = projections::monsters(seed.0);

    for (PreGameSpawnPoint { id }, transform) in &spawn_points {
        #[cfg(target = "wasm32")]
        web_sys::console::log_1(&JsValue::from_str(&format!("Found spawn point {:#?}", id)));

        let monster = monsters[*id as usize - 1];

        commands.spawn((
            MonsterBundle {
                monster: Monster::Idle,
                speed: monster::Speed(5.0),
                stats: monster::Stats {
                    ..Default::default()
                },
                ..default()
            },
            *transform,
            BlueprintName(monster.blueprint_name.to_owned()),
            SpawnHere,
        ));
    }
}
