#![allow(clippy::type_complexity)]
use bevy_tweening::{lens::TransformPositionLens, Animator, Delay, EaseFunction, Tween};
use std::time::Duration;

use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_gltf_blueprints::{AnimationPlayerLink, Animations};

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Monster>()
            .add_systems(Update, init_animation)
            .add_systems(Update, run_timers);
    }
}

#[derive(Component, Default)]
pub struct Start;

#[derive(Bundle, Default)]
pub struct MonsterBundle {
    pub monster: Monster,
    // pub scene: SceneBundle,
    pub speed: Speed,
    pub stats: Stats,
    // pub animations: NamedAnimations,
    // pub behaviour: StateMachine<Behaviour>,
    pub behaviour_timer: BehaviourTimer,
    pub start: Start,
}

#[derive(Debug, Reflect, Default)]
pub struct TimedAnimation {
    handle: Handle<AnimationClip>,
    duration: f32,
}

#[derive(Debug, Reflect, Component, Default)]
pub struct NamedAnimations {
    pub idle: TimedAnimation,
    pub jump: TimedAnimation,
    pub dance: TimedAnimation,
    pub death: TimedAnimation,
}

fn extract_animations(
    animation_map: &HashMap<String, Handle<AnimationClip>>,
    animations: &Res<Assets<AnimationClip>>,
) -> NamedAnimations {
    let get_timed_animation = |name: &str, alt_name: &str| {
        let handle = animation_map
            .get(name)
            .or_else(|| animation_map.get(alt_name))
            .expect("failed to find animation");
        let duration = animations.get(handle).unwrap().duration();

        TimedAnimation {
            handle: handle.clone(),
            duration,
        }
    };

    NamedAnimations {
        idle: get_timed_animation("CharacterArmature|Idle", "RobotArmature|Idle"),
        jump: get_timed_animation("CharacterArmature|Jump", "RobotArmature|Jump"),
        dance: get_timed_animation("CharacterArmature|Dance", "RobotArmature|Dance"),
        death: get_timed_animation("CharacterArmature|Death", "RobotArmature|Death"),
    }
}
// 0.83333333
// 0.41666666

#[derive(Component, Debug, Reflect, Default)]
pub struct Stats {
    pub recovery_time: f32,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct BehaviourTimer {
    timer: Timer,
    next_state: Monster,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct Speed(pub f32);

#[derive(Debug, Clone, Default, Copy, Component, Reflect, PartialEq)]
pub enum Monster {
    #[default]
    Idle,
    Jumping,
    Recovering,
    Dancing,
    Dead,
}

pub fn init_animation(
    mut commands: Commands,
    query: Query<(Entity, &Animations, &Monster), With<Start>>,
    clips: Res<Assets<AnimationClip>>,
) {
    for (entity, animations, monster) in &query {
        let named_animations = extract_animations(&animations.named_animations, &clips);

        commands
            .entity(entity)
            .insert(named_animations)
            .remove::<Start>()
            .insert(BehaviourTimer {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
                next_state: *monster,
            });
    }
}

#[derive(Debug, Component, Reflect)]
pub struct JumpTarget {
    pub start: Vec3,
    pub end: Vec3,
}

pub fn run_timers(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &AnimationPlayerLink,
        &NamedAnimations,
        &mut Monster,
        &mut BehaviourTimer,
        &Transform,
    )>,
    mut anim_players: Query<&mut AnimationPlayer>,
    time: Res<Time>,
) {
    for (entity, anim_link, animations, mut monster, mut timer, transform) in &mut query {
        if timer.timer.finished() {
            continue;
        }

        timer.timer.tick(time.delta());
        if !timer.timer.finished() {
            continue;
        }

        let mut player = anim_players.get_mut(anim_link.0).unwrap();

        *monster = timer.next_state;
        match *monster {
            Monster::Idle => {
                player
                    .play_with_transition(
                        animations.idle.handle.clone(),
                        Duration::from_secs_f32(0.1),
                    )
                    .repeat();
            }
            Monster::Jumping => {
                let target = transform.translation + transform.back() * 0.7;

                let tween = Delay::new(Duration::from_secs_f32(0.25)).then(Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_secs_f32(animations.jump.duration * 2.0 - 0.25),
                    TransformPositionLens {
                        start: transform.translation,
                        end: target,
                    },
                ));

                commands.entity(entity).insert(Animator::new(tween));

                player
                    .play_with_transition(
                        animations.jump.handle.clone(),
                        Duration::from_secs_f32(0.1),
                    )
                    .set_speed(0.5);

                *timer = BehaviourTimer {
                    timer: Timer::from_seconds(
                        animations.jump.duration * 2.0 - 0.2,
                        TimerMode::Once,
                    ),
                    next_state: Monster::Recovering,
                };
            }
            Monster::Recovering => {
                player.play_with_transition(
                    animations.idle.handle.clone(),
                    Duration::from_secs_f32(0.2),
                );

                let recovery = 0.2 + (0.8333 - animations.jump.duration);

                *timer = BehaviourTimer {
                    timer: Timer::from_seconds(recovery, TimerMode::Once),
                    next_state: Monster::Jumping,
                };
            }
            Monster::Dancing => {
                player
                    .play_with_transition(
                        animations.dance.handle.clone(),
                        Duration::from_secs_f32(0.1),
                    )
                    .repeat();
            }
            Monster::Dead => {
                player.play_with_transition(
                    animations.death.handle.clone(),
                    Duration::from_secs_f32(0.1),
                );
            }
        }
    }
}
