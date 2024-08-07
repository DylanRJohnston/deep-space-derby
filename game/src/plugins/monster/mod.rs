#![allow(clippy::type_complexity)]
use bevy_tweening::{lens::TransformPositionLens, Animator, Delay, EaseFunction, Tween};
use rand::{distributions::Uniform, thread_rng, Rng};
use shared::models::monsters::Monster;
use std::time::Duration;

use bevy::prelude::*;

use super::{
    animation_link::{AnimationLink, AnimationRoot},
    scenes::{race::RaceTimer, GameAssets},
};

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MonsterBehaviour>()
            .add_systems(Update, init_animation)
            .add_systems(Update, run_timers)
            .observe(spawn_monster)
            .observe(despawn_all_monsters);
    }
}

#[derive(Component, Default)]
pub struct Start;

#[derive(Bundle, Default)]
pub struct MonsterBundle {
    pub id: MonsterID,
    pub monster: MonsterBehaviour,
    // pub scene: SceneBundle,
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
    pub timer: Timer,
    pub next_state: MonsterBehaviour,
}

#[derive(Debug, Clone, Default, Copy, Component, Reflect, PartialEq)]
pub enum MonsterBehaviour {
    #[default]
    Idle,
    Jumping(f32),
    Recovering,
    Dancing,
    Dead,
}

pub fn init_animation(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    clips: Res<Assets<AnimationClip>>,
    new_monsters: Query<
        (Entity, &AnimationLink, &Handle<Gltf>),
        (With<MonsterBehaviour>, With<Start>),
    >,
    gltfs: Res<Assets<Gltf>>,
) {
    for (entity, animation_player_link, gltf_handle) in &new_monsters {
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
                next_state: MonsterBehaviour::Idle,
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
        &mut MonsterBehaviour,
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
            MonsterBehaviour::Idle => {
                transition
                    .play(
                        &mut player,
                        animations.idle.index,
                        Duration::from_secs_f32(0.1),
                    )
                    .set_speed(thread_rng().sample(Uniform::new(0.9, 1.1)))
                    .repeat();
            }
            MonsterBehaviour::Jumping(amount) => {
                let jump_delay = 0.2;
                let animation_speed = 0.75;
                let duration = animations.jump.duration * (1.0 / animation_speed) - jump_delay;
                let stage_distance = 0.75;

                let target = transform.translation + transform.back() * stage_distance * amount;

                let tween = Delay::new(Duration::from_secs_f32(jump_delay)).then(Tween::new(
                    EaseFunction::QuadraticOut,
                    Duration::from_secs_f32(duration),
                    TransformPositionLens {
                        start: transform.translation,
                        end: target,
                    },
                ));

                commands.entity(entity).insert(Animator::new(tween));

                transition
                    .play(
                        &mut player,
                        animations.jump.index,
                        Duration::from_secs_f32(0.2),
                    )
                    .set_speed(animation_speed);

                *timer = BehaviourTimer {
                    timer: Timer::from_seconds(duration, TimerMode::Once),
                    next_state: MonsterBehaviour::Recovering,
                };
            }
            MonsterBehaviour::Recovering => {
                transition
                    .play(
                        &mut player,
                        animations.idle.index,
                        Duration::from_secs_f32(0.2),
                    )
                    .repeat();
            }
            MonsterBehaviour::Dancing => {
                transition
                    .play(
                        &mut player,
                        animations.dance.index,
                        Duration::from_secs_f32(0.3),
                    )
                    .set_speed(thread_rng().sample(Uniform::new(0.9, 1.1)))
                    .repeat();
            }
            MonsterBehaviour::Dead => {
                transition.play(
                    &mut player,
                    animations.death.index,
                    Duration::from_secs_f32(0.1),
                );
            }
        }
    }
}

#[derive(Debug, Event)]
pub struct DespawnAllMonsters;

fn despawn_all_monsters(
    _trigger: Trigger<DespawnAllMonsters>,
    mut commands: Commands,
    monsters: Query<Entity, With<MonsterBehaviour>>,
) {
    for monster in &monsters {
        commands.entity(monster).despawn_recursive();
    }
}

#[derive(Debug, Event)]
pub struct SpawnMonster {
    pub transform: Transform,
    pub monster: &'static Monster,
    pub id: usize,
}

#[derive(Debug, Component, Default, Deref)]
pub struct MonsterID(usize);

fn spawn_monster(
    trigger: Trigger<SpawnMonster>,
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    game_assets: Option<Res<GameAssets>>,
) {
    let SpawnMonster {
        transform,
        monster,
        id,
    } = trigger.event();

    let handle = game_assets
        .as_ref()
        .ok_or("game assets haven't loaded yet")
        .unwrap()
        .models
        .get(monster.blueprint_name)
        .ok_or_else(|| {
            format!(
                "failed to find asset for monster: {}, available models: {:?}",
                monster.blueprint_name,
                game_assets.as_ref().unwrap().models.keys()
            )
        })
        .unwrap();

    let scene = gltfs
        .get(handle)
        .ok_or_else(|| {
            format!(
                "failed to retrieve asset for monster: {}",
                monster.blueprint_name
            )
        })
        .unwrap();

    let mut transform = *transform;
    transform.scale = Vec3::splat(0.25);

    commands.spawn((
        Name::from(monster.name),
        MonsterBundle {
            id: MonsterID(*id),
            monster: MonsterBehaviour::default(),
            behaviour_timer: BehaviourTimer {
                timer: Timer::from_seconds(0.1, TimerMode::Once),
                next_state: MonsterBehaviour::Idle,
            },
            ..default()
        },
        RaceTimer::default(),
        handle.clone(),
        SceneBundle {
            scene: scene.scenes[0].clone(),
            transform,
            ..default()
        },
    ));
}
