use std::time::Duration;

use bevy::prelude::*;
use bevy_gltf_blueprints::{AnimationPlayerLink, Animations};
use rand::{
    distributions::{Bernoulli, Uniform},
    thread_rng, Rng,
};

pub struct SpectatorPlugin;

impl Plugin for SpectatorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Spectator>()
            .add_systems(Update, init_animation);
    }
}

#[derive(Debug, Clone, Component, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
pub struct Spectator;

#[derive(Debug, Component)]
pub struct BehaviourTimer(Timer);

pub fn init_animation(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &AnimationPlayerLink,
        &Animations,
        &Spectator,
        Option<&mut BehaviourTimer>,
    )>,
    mut players: Query<&mut AnimationPlayer>,
    clips: Res<Assets<AnimationClip>>,
    time: Res<Time>,
) {
    for (entity, player_link, animations, _spectator, mut behaviour) in &mut query {
        let mut player = players.get_mut(player_link.0).unwrap();

        match behaviour.as_deref_mut() {
            None => {
                commands.entity(entity).remove::<Spectator>();
                // .insert(BehaviourTimer(Timer::from_seconds(
                //     thread_rng().sample(Uniform::new(10000.0, 10001.0)),
                //     TimerMode::Once,
                // )));

                player
                    .start(
                        animations
                            .named_animations
                            .get("Armature|mixamo.com|Layer0")
                            .unwrap()
                            .clone(),
                    )
                    .set_speed(thread_rng().sample(Uniform::new(0.9, 1.1)))
                    .repeat();
            }
            Some(BehaviourTimer(timer)) => {
                timer.tick(time.delta());

                if !timer.finished() {
                    continue;
                }

                match thread_rng().sample(Bernoulli::new(0.5).unwrap()) {
                    true => {
                        let handle = animations
                            .named_animations
                            .get("CharacterArmature|Wave")
                            .unwrap();

                        let duration = clips.get(handle).unwrap().duration();

                        player.start_with_transition(handle.clone(), Duration::from_secs_f32(0.1));

                        *timer =
                            Timer::new(Duration::from_secs_f32(duration - 0.1), TimerMode::Once);
                    }
                    false => {
                        let handle = animations
                            .named_animations
                            .get("CharacterArmature|Idle_Gun")
                            .unwrap();

                        let duration = clips.get(handle).unwrap().duration();

                        player.start_with_transition(handle.clone(), Duration::from_secs_f32(0.1));

                        *timer =
                            Timer::new(Duration::from_secs_f32(duration - 0.1), TimerMode::Once);
                    }
                }
            }
        }
    }
}

