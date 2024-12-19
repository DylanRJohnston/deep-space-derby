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

#[derive(Debug, Clone, Component, Deref)]
pub struct DanceAnimation(Handle<AnimationClip>);

#[derive(Clone, Bundle)]
pub struct SpectatorBundle {
    pub scene: SceneRoot,
    pub spectator: Spectator,
    pub animation_root: AnimationRoot,
    pub dance: DanceAnimation,
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

        commands.entity(entity).insert((
            SpectatorBundle {
                scene: SceneRoot(model.scenes[0].clone()),
                animation_root: AnimationRoot,
                spectator: Spectator,
                dance: DanceAnimation(
                    model
                        .named_animations
                        .get("Armature|mixamo.com|Layer0")
                        .unwrap()
                        .clone(),
                ),
            },
            transform,
        ));
    }
}

pub fn init_animation(
    mut commands: Commands,
    mut query: Query<(&DanceAnimation, &AnimationLink), (With<Spectator>, Added<AnimationLink>)>,
    mut players: Query<&mut AnimationPlayer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (dance, animation_link) in &mut query {
        let mut player = players.get_mut(animation_link.0).unwrap();

        let (graph, animation_index) = AnimationGraph::from_clip(dance.0.clone());
        let graph_handle = graphs.add(graph);

        commands
            .entity(animation_link.0)
            .insert(AnimationGraphHandle(graph_handle));

        player
            .play(animation_index)
            .set_speed(thread_rng().sample(Uniform::new(0.9, 1.1)))
            .repeat();
    }
}
