use bevy::{
    prelude::*,
    render::{
        mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
        primitives::Aabb,
    },
};

pub struct SkinnedMeshPlugin;

impl Plugin for SkinnedMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            |inverse_bindposes: Res<Assets<SkinnedMeshInverseBindposes>>,
             mut query: Query<(&SkinnedMesh, &mut Aabb), Added<Aabb>>| {
                // HACK:
                // for (skinned_mesh, mut aabb) in query.iter_mut() {
                //     println!("Found skinned mesh");

                //     let Some(inverse_bindposes) =
                //         inverse_bindposes.get(&skinned_mesh.inverse_bindposes)
                //     else {
                //         continue;
                //     };

                //     println!("Found inverse bindposes");

                //     // let inverse_bindpose = inverse_bindposes[0]; // `0` probably won't work in all cases

                //     // // multiplying by `inverse_bindpose` seems to be the standard (https://github.com/KhronosGroup/glTF-Blender-IO/issues/1887)
                //     // aabb.center = (inverse_bindpose * aabb.center.extend(0.0)).into();
                //     // aabb.half_extents = (inverse_bindpose * aabb.half_extents.extend(0.0)).into();
                // }
            },
        );
    }
}
