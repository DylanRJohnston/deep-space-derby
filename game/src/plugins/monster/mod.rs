#![allow(clippy::type_complexity)]
use bevy_tweening::{lens::TransformPositionLens, Animator, Delay, EaseFunction, Tween};
use rand::{distributions::Uniform, thread_rng, Rng};
use std::time::Duration;

use bevy::prelude::*;

use super::animation_link::{AnimationLink, AnimationRoot};

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
    pub animation_root: AnimationRoot,
}

#[derive(Debug, Reflect, Default)]
pub struct TimedAnimation {
    index: AnimationNodeIndex,
    duration: f32,
}

#[derive(Debug, Reflect, Component, Default)]
pub struct NamedAnimations {
    pub idle: TimedAnimation,
    pub jump: TimedAnimation,
    pub dance: TimedAnimation,
    pub death: TimedAnimation,
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

impl From<i32> for Speed {
    fn from(value: i32) -> Self {
        Self(value as f32)
    }
}

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
    mut graphs: ResMut<Assets<AnimationGraph>>,
    clips: Res<Assets<AnimationClip>>,
    new_monsters: Query<(Entity, &Monster, &AnimationLink, &Handle<Gltf>), With<Start>>,
    gltfs: Res<Assets<Gltf>>,
) {
    for (entity, monster, animation_player_link, gltf_handle) in &new_monsters {
        let gltf = gltfs.get(gltf_handle).unwrap();

        let mut graph = AnimationGraph::new();

        let mut get_timed_animation = |name: &str, alt_name: &str| {
            let handle = gltf
                .named_animations
                .get(name)
                .or_else(|| gltf.named_animations.get(alt_name))
                .expect("failed to find animation")
                .clone();

            let duration = clips.get(&handle).unwrap().duration();

            let index = graph.add_clip(handle, 1.0, AnimationNodeIndex::new(0));

            TimedAnimation { index, duration }
        };

        commands
            .entity(entity)
            .insert((NamedAnimations {
                idle: get_timed_animation("CharacterArmature|Idle", "RobotArmature|Idle"),
                jump: get_timed_animation("CharacterArmature|Jump", "RobotArmature|Jump"),
                dance: get_timed_animation("CharacterArmature|Dance", "RobotArmature|Dance"),
                death: get_timed_animation("CharacterArmature|Death", "RobotArmature|Death"),
            },))
            .remove::<Start>()
            .insert(BehaviourTimer {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
                next_state: Monster::Idle,
            });

        let graph_handle = graphs.add(graph);
        let transition = AnimationTransitions::new();

        // gltf loads the animation as a child node, the animation graph must be on the same entity as the animation player
        commands
            .entity(animation_player_link.0)
            .insert((graph_handle, transition));
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
        &AnimationLink,
        &NamedAnimations,
        &mut Monster,
        &mut BehaviourTimer,
        &Transform,
    )>,
    mut anim_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
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

        let (mut player, mut transition) = anim_players.get_mut(anim_link.0).unwrap();

        *monster = timer.next_state;
        match *monster {
            Monster::Idle => {
                transition
                    .play(
                        &mut player,
                        animations.idle.index,
                        Duration::from_secs_f32(0.1),
                    )
                    .set_speed(thread_rng().sample(Uniform::new(0.9, 1.1)))
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

                transition.play(
                    &mut player,
                    animations.jump.index,
                    Duration::from_secs_f32(0.1),
                );

                *timer = BehaviourTimer {
                    timer: Timer::from_seconds(
                        animations.jump.duration * 2.0 - 0.2,
                        TimerMode::Once,
                    ),
                    next_state: Monster::Recovering,
                };
            }
            Monster::Recovering => {
                transition.play(
                    &mut player,
                    animations.idle.index,
                    Duration::from_secs_f32(0.2),
                );

                let recovery = 0.2 + (0.8333 - animations.jump.duration);

                *timer = BehaviourTimer {
                    timer: Timer::from_seconds(recovery, TimerMode::Once),
                    next_state: Monster::Jumping,
                };
            }
            Monster::Dancing => {
                transition
                    .play(
                        &mut player,
                        animations.dance.index,
                        Duration::from_secs_f32(0.1),
                    )
                    .set_speed(thread_rng().sample(Uniform::new(0.9, 1.1)))
                    .repeat();
            }
            Monster::Dead => {
                transition.play(
                    &mut player,
                    animations.death.index,
                    Duration::from_secs_f32(0.1),
                );
            }
        }
    }
}
