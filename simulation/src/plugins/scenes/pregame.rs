use bevy::prelude::*;

use super::SceneMetadata;

pub struct PreGamePlugin;

impl Plugin for PreGamePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PreGameSpawnPoint>()
            .register_type::<PreGameCamera>()
            .add_systems(Update, spawn_pregame_spawn_point_on_scene_load);
    }
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct PreGameSpawnPoint {
    pub id: u32,
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
                        id: n.as_u64().unwrap() as u32,
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
