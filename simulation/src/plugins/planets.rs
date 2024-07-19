use bevy::prelude::*;

use super::scenes::SceneMetadata;

pub struct PlanetsPlugin;

impl Plugin for PlanetsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RotateSpeed>()
            .register_type::<OrbitPoint>()
            .add_systems(Update, rotate)
            .add_systems(Update, orbit)
            .add_systems(Update, add_components_on_scene_load);
    }
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct RotateSpeed {
    pub speed: f32,
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct OrbitPoint {
    pub velocity: f32,
    pub point: Vec3,
}

pub fn rotate(mut query: Query<(&mut Transform, &RotateSpeed)>, time: Res<Time>) {
    for (mut transform, rotation) in &mut query {
        transform.rotate_y(time.delta_seconds() * rotation.speed);
    }
}

pub fn orbit(mut query: Query<(&mut Transform, &OrbitPoint)>, time: Res<Time>) {
    for (mut transform, orbit) in &mut query {
        transform.translate_around(
            orbit.point,
            Quat::from_rotation_y(time.delta_seconds() * orbit.velocity),
        )
    }
}

pub fn add_components_on_scene_load(
    query: Query<(Entity, &SceneMetadata), Added<SceneMetadata>>,
    named_entities: Query<(&Name, &Transform)>,
    mut commands: Commands,
) {
    for (entity, metadata) in &query {
        if let Some(value) = metadata.0.get("RotateSpeed") {
            match value {
                serde_json::Value::Number(n) if n.is_f64() => {
                    commands.entity(entity).insert(RotateSpeed {
                        speed: n.as_f64().unwrap() as f32,
                    });
                }
                other => panic!("RotateSpeed must be a f64, got {:?}", other),
            }
        }

        if let (Some(name), Some(speed)) =
            (metadata.0.get("OrbitAround"), metadata.0.get("OrbitSpeed"))
        {
            let name = match name {
                serde_json::Value::String(s) => s.clone(),
                other => panic!("OrbitAround must be a string, got {:?}", other),
            };

            let speed = match speed {
                serde_json::Value::Number(n) if n.is_f64() => n.as_f64().unwrap() as f32,
                other => panic!("OrbitSpeed must be a f64, got {:?}", other),
            };

            // Linear traversal of all entities, but there aren't that many... for now
            let planet = named_entities
                .into_iter()
                .find(|(planet_name, _)| planet_name.as_str() == name)
                .ok_or("failed to find planet to orbit around")
                .unwrap();

            commands.entity(entity).insert(OrbitPoint {
                velocity: speed,
                point: planet.1.translation,
            });
        }
    }
}
