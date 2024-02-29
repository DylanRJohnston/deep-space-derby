use bevy::prelude::*;

pub struct PlanetsPlugin;

impl Plugin for PlanetsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RotateSpeed>()
            .register_type::<OrbitPoint>()
            .add_systems(Update, rotate)
            .add_systems(Update, orbit);
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

