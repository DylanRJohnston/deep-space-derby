use bevy::prelude::*;

/// AnimationLinkPlugin solves an awkward ergonimics problem in bevy
/// with the way it's AnimationPlayer works when loading scenes.
/// The animation player is a child entity of the root scene.
/// This plugin automatically adds a new component [AnimationLink] to the
/// scene once the AnimationPlayer has loaded.
pub struct AnimationLinkPlugin;

impl Plugin for AnimationLinkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animation_link);
    }
}

#[derive(Component, Debug, Reflect)]
pub struct AnimationLink(pub Entity);

fn get_root(initial_entity: Entity, all_parents: &Query<&Parent>) -> Entity {
    let mut entity = initial_entity;

    while let Ok(parent) = all_parents.get(entity) {
        entity = parent.get();
    }

    entity
}

fn animation_link(
    mut commands: Commands,
    all_entities_with_parents: Query<&Parent>,
    players: Query<Entity, Added<AnimationPlayer>>,
) {
    for entity in &players {
        let root = get_root(entity, &all_entities_with_parents);

        commands.entity(root).insert(AnimationLink(entity));
    }
}
