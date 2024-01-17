#![allow(clippy::type_complexity)]
use statig::prelude::*;
use std::time::Duration;

use bevy::{gltf::Gltf, prelude::*, utils::hashbrown::HashMap};

use crate::AppState;

use super::{animation_link::AnimationLink, asset_loader::AssetPack};

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup)
            .add_systems(Update, run_behaviour)
            .add_systems(Update, move_forward)
            .add_systems(OnEnter(AppState::InGame), start_race);
    }
}

#[derive(Bundle, Default)]
pub struct MonsterBundle {
    pub scene: SceneBundle,
    pub speed: Speed,
    pub stats: Stats,
    pub animations: NamedAnimations,
    pub behaviour: StateMachine<Behaviour>,
    pub behaviour_timer: BehaviourTimer,
}

#[derive(Debug, Reflect, Component, Default)]
pub struct NamedAnimations {
    pub idle: Handle<AnimationClip>,
    pub jump: Handle<AnimationClip>,
    pub dance: Handle<AnimationClip>,
    pub death: Handle<AnimationClip>,
}

fn extract_animations(animation_map: &HashMap<String, Handle<AnimationClip>>) -> NamedAnimations {
    NamedAnimations {
        idle: animation_map
            .get("CharacterArmature|Idle")
            .expect("failed to find animation Idle")
            .clone(),
        jump: animation_map
            .get("CharacterArmature|Jump")
            .expect("failed to find animation Jump")
            .clone(),
        dance: animation_map
            .get("CharacterArmature|Dance")
            .expect("failed to find animation Dance")
            .clone(),
        death: animation_map
            .get("CharacterArmature|Death")
            .expect("failed to find animation Death")
            .clone(),
    }
}

fn setup(mut commands: Commands, asset_pack: Res<AssetPack>, assets: Res<Assets<Gltf>>) {
    let mushnub = assets.get(&asset_pack.mushnub).unwrap();

    commands.spawn(MonsterBundle {
        scene: SceneBundle {
            scene: mushnub.named_scenes["Root Scene"].clone(),
            transform: Transform::from_xyz(0.0, 0.0, -5.0),
            ..default()
        },
        speed: Speed(5.0),
        stats: Stats { recovery_time: 5.0 },
        animations: extract_animations(&mushnub.named_animations),
        behaviour: Behaviour.state_machine(),
        ..default()
    });

    let alien = assets.get(&asset_pack.alien).unwrap();

    commands.spawn(MonsterBundle {
        scene: SceneBundle {
            scene: alien.named_scenes["Root Scene"].clone(),
            transform: Transform::from_xyz(5.0, 0.0, -5.0),
            ..default()
        },
        speed: Speed(5.0),
        stats: Stats { recovery_time: 5.0 },
        animations: extract_animations(&alien.named_animations),
        behaviour: Behaviour.state_machine(),
        ..default()
    });

    let cactoro = assets.get(&asset_pack.cactoro).unwrap();

    commands.spawn(MonsterBundle {
        scene: SceneBundle {
            scene: cactoro.named_scenes["Root Scene"].clone(),
            transform: Transform::from_xyz(-5.0, 0.0, -5.0),
            ..default()
        },
        speed: Speed(5.0),
        stats: Stats { recovery_time: 5.0 },
        animations: extract_animations(&cactoro.named_animations),
        behaviour: Behaviour.state_machine(),
        ..default()
    });
}

#[derive(Component, Debug, Reflect, Default)]
pub struct Stats {
    pub recovery_time: f32,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct BehaviourTimer(Timer);

#[derive(Component, Debug, Reflect, Default)]
pub struct Speed(pub f32);
fn move_forward(
    mut query: Query<(&mut Transform, &Speed, &StateMachine<Behaviour>)>,
    time: Res<Time>,
) {
    for (mut transform, speed, state) in &mut query {
        match state.state() {
            State::Jumping {} => {}
            _ => {
                continue;
            }
        }

        transform.translation =
            transform.translation + transform.back() * speed.0 * time.delta_seconds()
    }
}

#[derive(Default)]
pub struct Behaviour;

#[derive(Debug)]
pub enum InputEvent {
    TimerElapsed,
    Next,
}

pub struct Context<'a> {
    pub animations: &'a NamedAnimations,
    pub player: &'a mut AnimationPlayer,
    pub timer: &'a mut Timer,
}

// type Context<'a, 'b> = EventWriter<'a, OutputEvent>;

#[state_machine(initial = "State::idle()", context_identifier = "_context")]
impl Behaviour {
    #[state(entry_action = "enter_idle")]
    fn idle(_context: &mut Context<'_>, event: &InputEvent) -> Response<State> {
        match event {
            InputEvent::Next => Transition(State::jumping()),
            _ => Super,
        }
    }

    #[state(entry_action = "enter_dancing")]
    fn dancing(event: &InputEvent) -> Response<State> {
        match event {
            InputEvent::Next => Transition(State::dead()),
            _ => Super,
        }
    }

    #[superstate]
    fn running(event: &InputEvent) -> Response<State> {
        match event {
            InputEvent::Next => Transition(State::dancing()),
            _ => Super,
        }
    }

    #[allow(unused_variables)]
    #[state(entry_action = "enter_dead")]
    fn dead(event: &InputEvent) -> Response<State> {
        Handled
    }

    #[state(superstate = "running", entry_action = "enter_jumping")]
    fn jumping(&mut self, event: &InputEvent) -> Response<State> {
        match event {
            InputEvent::TimerElapsed => Transition(State::recovering()),
            _ => Super,
        }
    }

    #[state(superstate = "running", entry_action = "enter_recovering")]
    fn recovering(event: &InputEvent) -> Response<State> {
        match event {
            InputEvent::TimerElapsed => Transition(State::jumping()),
            _ => Super,
        }
    }

    #[action]
    fn enter_jumping(&mut self, _context: &mut Context) {
        let Context {
            animations,
            player,
            timer,
            ..
        } = _context;

        player.start(animations.jump.clone());

        **timer = Timer::from_seconds(0.3, TimerMode::Once);
    }

    #[action]
    fn enter_recovering(&mut self, _context: &mut Context) {
        let Context {
            animations,
            player,
            timer,
            ..
        } = _context;

        player
            .start_with_transition(animations.idle.clone(), Duration::from_secs_f32(0.2))
            .repeat();

        let recovery = 0.1 + rand::random::<f32>() * 3.0;

        **timer = Timer::from_seconds(recovery, TimerMode::Once);
    }

    #[action]
    fn enter_idle(_context: &mut Context) {
        println!("Entering Idle Action Controller");

        let Context {
            animations, player, ..
        } = _context;

        player
            .start_with_transition(animations.idle.clone(), Duration::from_secs_f32(0.2))
            .repeat();
    }

    #[action]
    fn enter_dancing(_context: &mut Context) {
        let Context {
            animations, player, ..
        } = _context;

        player
            .play_with_transition(animations.dance.clone(), Duration::from_secs_f32(1.0))
            .repeat();
    }

    #[action]
    fn enter_dead(_context: &mut Context) {
        let Context {
            animations, player, ..
        } = _context;

        player.start(animations.death.clone());
    }
}

fn run_behaviour(
    mut query: Query<(
        &AnimationLink,
        &NamedAnimations,
        &mut StateMachine<Behaviour>,
        &mut BehaviourTimer,
    )>,
    mut animation_players: Query<&mut AnimationPlayer>,
    time: Res<Time>,
) {
    for (animation_link, animations, mut machine, mut behaviour_timer) in &mut query {
        let player = &mut animation_players
            .get_mut(animation_link.0)
            .expect("animation link refers to removed animation player");

        if behaviour_timer.0.finished() {
            continue;
        }

        behaviour_timer.0.tick(time.delta());

        if !behaviour_timer.0.finished() {
            continue;
        };

        machine.handle_with_context(
            &InputEvent::TimerElapsed,
            &mut Context {
                animations,
                player,
                timer: &mut behaviour_timer.0,
            },
        );
    }
}

fn start_race(
    mut query: Query<(
        &AnimationLink,
        &NamedAnimations,
        &mut StateMachine<Behaviour>,
        &mut BehaviourTimer,
    )>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for (animation_link, animations, mut machine, mut behaviour_timer) in &mut query {
        let player = &mut animation_players
            .get_mut(animation_link.0)
            .expect("animation link refers to removed animation player");

        machine.handle_with_context(
            &InputEvent::Next,
            &mut Context {
                animations,
                player,
                timer: &mut behaviour_timer.0,
            },
        );
    }
}
