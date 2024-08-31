#![allow(clippy::type_complexity)]
use bevy_tweening::{lens::TransformPositionLens, Animator, Delay, EaseFunction, Tween};
use rand::{distributions::Uniform, thread_rng, Rng};
use shared::models::{monsters::Monster, projections::Jump};
use std::time::Duration;

use bevy::prelude::*;

use super::{
    animation_link::{AnimationLink, AnimationRoot},
    scenes::{race::RaceTimer, GameAssets},
};

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, init_animation)
            .add_systems(Update, run_timers)
            .observe(spawn_monster)
            .observe(despawn_all_monsters);
    }
}

#[derive(Component, Default)]
pub struct Start(Transform);

#[derive(Bundle, Default)]
pub struct MonsterBundle {
    pub id: MonsterID,
    pub monster: MonsterBehaviour,
    // pub scene: SceneBundle,
    pub stats: Stats,
    // pub animations: NamedAnimations,
    // pub behaviour: StateMachine<Behaviour>,
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

#[derive(Debug, Clone, Default, Copy, Component, PartialEq)]
pub enum MonsterBehaviour {
    #[default]
    Idle,
    Jumping(Jump),
    Recovering,
    Dancing,
    Dead,
}

pub fn init_animation(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    clips: Res<Assets<AnimationClip>>,
    new_monsters: Query<(Entity, &AnimationLink, &Handle<Gltf>), Added<AnimationLink>>,
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

        commands.entity(entity).insert((NamedAnimations {
            idle: get_timed_animation("CharacterArmature|Idle", "RobotArmature|Idle"),
            jump: get_timed_animation("CharacterArmature|Jump", "RobotArmature|Jump"),
            dance: get_timed_animation("CharacterArmature|Dance", "RobotArmature|Dance"),
            death: get_timed_animation("CharacterArmature|Death", "RobotArmature|Death"),
        },));

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
    mut query: Query<
        (
            Entity,
            &Start,
            &AnimationLink,
            &NamedAnimations,
            &MonsterBehaviour,
            &Transform,
        ),
        Or<(Changed<MonsterBehaviour>, Added<NamedAnimations>)>,
    >,
    mut anim_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (entity, start, anim_link, animations, monster, transform) in &mut query {
        let (mut player, mut transition) = anim_players.get_mut(anim_link.0).unwrap();

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
            MonsterBehaviour::Jumping(jump) => {
                let duration = jump.end - jump.start;
                let animation_speed = animations.jump.duration / duration;
                let jump_delay = 0.3 * duration;
                let jump_end = 0.1 * duration;

                let stage_distance = 0.85;
                let target =
                    start.0.translation + transform.back() * stage_distance * jump.distance;

                let tween = Delay::new(Duration::from_secs_f32(jump_delay)).then(Tween::new(
                    EaseFunction::QuadraticOut,
                    Duration::from_secs_f32(duration - jump_delay - jump_end),
                    TransformPositionLens {
                        start: transform.translation,
                        end: target,
                    },
                ));

                commands.entity(entity).insert(Animator::new(tween));

                // transition
                //     .play(&mut player, animations.jump.index, Duration::ZERO)
                //     .set_speed(animation_speed)
                //     .replay();

                player
                    .start(animations.jump.index)
                    .set_speed(animation_speed)
                    .replay();
            }
            MonsterBehaviour::Recovering => {}
            MonsterBehaviour::Dancing => {
                transition
                    .play(
                        &mut player,
                        animations.dance.index,
                        Duration::from_secs_f32(1.0),
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

#[derive(Debug, Event, Default)]
pub struct SpawnMonster {
    pub transform: Transform,
    pub start: Option<Transform>,
    pub monster: Monster,
    pub behaviour: MonsterBehaviour,
    pub id: usize,
}

#[derive(Debug, Component, Default, Deref)]
pub struct MonsterID(pub usize);

#[derive(Debug, Component, Deref)]
pub struct MonsterInfo(pub Monster);

fn spawn_monster(
    trigger: Trigger<SpawnMonster>,
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    game_assets: Option<Res<GameAssets>>,
) {
    let SpawnMonster {
        transform,
        start,
        monster,
        behaviour,
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
            monster: *behaviour,
            start: Start(start.unwrap_or(transform)),
            ..default()
        },
        MonsterInfo(*monster),
        RaceTimer::default(),
        handle.clone(),
        SceneBundle {
            scene: scene.scenes[0].clone(),
            transform,
            ..default()
        },
    ));
}
