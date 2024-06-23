use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::render::camera::Exposure;
use bevy::render::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::render::primitives::Aabb;
use bevy::{
    app::Startup,
    ecs::system::{Commands, ResMut},
    gizmos::{
        aabb::{AabbGizmoConfigGroup, AabbGizmoPlugin},
        config::GizmoConfigStore,
    },
};
use bevy_editor_pls::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_registry_export::*;
use simulation::{plugins::event_stream::Seed, start};

fn main() {
    start(|app| {
        app.add_plugins(ExportRegistryPlugin::default())
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(WorldInspectorPlugin::default());
        // .add_plugins(AabbGizmoPlugin)
        // .add_plugins(EditorPlugin::default());

        app
            // dont format me
            // .add_systems(Startup, |mut config_store: ResMut<GizmoConfigStore>| {
            //     config_store.config_mut::<AabbGizmoConfigGroup>().1.draw_all = true;
            // })
            .register_type::<Exposure>()
            .add_systems(Startup, |mut commands: Commands| {
                commands.insert_resource(Seed(2))
            });
        // .add_systems(
        //     Update,
        //     (|mut cmds: Commands, q_aabb: Query<Entity, With<Aabb>>| {
        //         println!("Removing bounding boxes");
        //         for e in &q_aabb {
        //             cmds.entity(e).remove::<Aabb>();
        //         }
        //     })
        //     .run_if(input_just_pressed(KeyCode::KeyL)),
        // )
    });
}
