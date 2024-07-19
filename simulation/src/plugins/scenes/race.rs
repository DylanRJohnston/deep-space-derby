use bevy::prelude::*;

use super::SceneMetadata;

pub struct RacePlugin;

impl Plugin for RacePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RaceSpawnPoint>()
            .register_type::<RaceStartCamera>()
            .add_systems(Update, spawn_race_spawn_point_on_scene_load);
    }
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct RaceSpawnPoint {
    pub id: u32,
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
                        id: n.as_u64().unwrap() as u32,
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
