use std::{hash::Hash, ops::Deref};

use bevy::{prelude::*, utils::HashMap};

/// AnimationLinkPlugin solves an awkward ergonimics problem in bevy
/// with the way it's AnimationPlayer works when loading scenes.
/// The animation player is a child entity of the root scene.
/// This plugin automatically adds a new component [AnimationLink] to the
/// scene once the AnimationPlayer has loaded.
pub struct AnimationLinkPlugin;

impl Plugin for AnimationLinkPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AnimationLink>()
            .add_systems(Update, animation_link);
    }
}

#[derive(Component, Debug, Reflect, Clone, Copy)]
pub struct AnimationLink(pub Entity);

#[derive(Component, Debug, Reflect, Clone, Copy)]
pub struct AnimationRoot;

#[derive(Component, Debug, Clone)]
pub struct NamedAnimations(pub HashMap<String, Handle<AnimationClip>>);

impl Deref for AnimationLink {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn get_player_entity(
    parent: Entity,
    all_children: &Query<(Entity, &Children, Option<&AnimationPlayer>)>,
) -> Option<Entity> {
    if let Ok((entity, children, player)) = all_children.get(parent) {
        if player.is_some() {
            return Some(entity);
        }

        for child in children.iter() {
            if let Some(player) = get_player_entity(*child, all_children) {
                return Some(player);
            }
        }
    }

    None
}

// Recursive descent is slower than ascent, but there appears to be a timing problem with the animation root component
fn animation_link(
    mut commands: Commands,
    all_entities_with_children: Query<(Entity, &Children, Option<&AnimationPlayer>)>,
    animation_roots: Query<Entity, Added<AnimationRoot>>,
) {
    for animation_root in &animation_roots {
        if let Some(player) = get_player_entity(animation_root, &all_entities_with_children) {
            commands
                .entity(animation_root)
                .insert(AnimationLink(player));
        }
    }
}
