use bevy::prelude::*;
use rand::{distributions::Uniform, thread_rng, Rng};

use super::{
    animation_link::{AnimationLink, AnimationRoot},
    scenes::{GameAssets, SceneMetadata},
};

pub struct SpectatorPlugin;

impl Plugin for SpectatorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Spectator>()
            .add_systems(Update, spawn_spectators_on_scene_load)
            .add_systems(Update, init_animation);
    }
}

#[derive(Debug, Clone, Component, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
pub struct Spectator;

#[derive(Clone, Bundle)]
pub struct SpectatorBundle {
    pub scene: SceneBundle,
    pub spectator: Spectator,
    pub animation_root: AnimationRoot,
    pub dance: Handle<AnimationClip>,
}

pub fn spawn_spectators_on_scene_load(
    query: Query<(Entity, &SceneMetadata, &Transform), Added<SceneMetadata>>,
    game_assets: Option<Res<GameAssets>>,
    gltfs: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    for (entity, metadata, transform) in &query {
        if metadata.0.get("Spectator").is_none() {
            continue;
        }

        let handle = game_assets
            .as_ref()
            .ok_or("game_assets not yet loaded")
            .unwrap()
            .models
            .get("library/Spectator.glb")
            .ok_or("missing spectator model")
            .unwrap();

        let model = gltfs
            .get(handle)
            .ok_or("spectator handle resolving to a missing asset")
            .unwrap();

        let mut transform = *transform;
        transform.scale = Vec3::splat(0.65);

        commands.entity(entity).insert(SpectatorBundle {
            scene: SceneBundle {
                scene: model.scenes[0].clone(),
                transform,
                ..default()
            },
            animation_root: AnimationRoot,
            spectator: Spectator,
            dance: model
                .named_animations
                .get("Armature|mixamo.com|Layer0")
                .unwrap()
                .clone(),
        });
    }
}

pub fn init_animation(
    mut commands: Commands,
    mut query: Query<
        (&Handle<AnimationClip>, &AnimationLink),
        (With<Spectator>, Added<AnimationLink>),
    >,
    mut players: Query<&mut AnimationPlayer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (dance, animation_link) in &mut query {
        println!("found spectator");

        let mut player = players.get_mut(animation_link.0).unwrap();

        let (graph, animation_index) = AnimationGraph::from_clip(dance.clone());
        let graph_handle = graphs.add(graph);

        commands.entity(animation_link.0).insert(graph_handle);

        player
            .play(animation_index)
            .set_speed(thread_rng().sample(Uniform::new(0.9, 1.1)))
            .repeat();
    }
}
