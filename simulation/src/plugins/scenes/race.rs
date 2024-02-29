use bevy::prelude::*;

pub struct RacePlugin;

impl Plugin for RacePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RaceSpawnPoint>()
            .register_type::<RaceStartCamera>();
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

