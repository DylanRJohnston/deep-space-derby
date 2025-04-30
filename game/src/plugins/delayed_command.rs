use std::time::Duration;

use bevy::{ecs::system::EntityCommands, prelude::*};

pub struct DelayedCommandPlugin;

impl Plugin for DelayedCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_delayed_commands);
    }
}

trait Thunk: FnOnce(&mut Commands) + Send + Sync + 'static {}
impl<T> Thunk for T where T: FnOnce(&mut Commands) + Send + Sync + 'static {}

#[derive(Component, Deref, DerefMut)]
pub struct DelayedCommand {
    #[deref]
    pub thunk: Option<Box<dyn Thunk>>,
    pub delay: Timer,
}

impl DelayedCommand {
    pub fn new(secs: f32, thunk: impl Thunk) -> Self {
        Self {
            thunk: Some(Box::new(thunk)),
            delay: Timer::new(Duration::from_secs_f32(secs), TimerMode::Once),
        }
    }
}

fn run_delayed_commands(
    mut commands: Commands,
    mut delayed_commands: Query<(Entity, &mut DelayedCommand)>,
    time: Res<Time>,
) {
    for (entity, mut delayed) in &mut delayed_commands {
        if !delayed.delay.tick(time.delta()).just_finished() {
            continue;
        }

        let Some(mut thunk) = delayed.take() else {
            continue;
        };

        (thunk)(&mut commands);
        commands.entity(entity).despawn_recursive();
    }
}

pub trait DelayedCommandExt {
    fn delayed(&mut self, secs: f32, thunk: impl Thunk) -> EntityCommands<'_>;
}

impl DelayedCommandExt for Commands<'_, '_> {
    fn delayed(&mut self, secs: f32, thunk: impl Thunk) -> EntityCommands<'_> {
        self.spawn(DelayedCommand::new(secs, thunk))
    }
}
