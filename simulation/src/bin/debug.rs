use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_registry_export::*;
use simulation::start;

fn main() {
    start(|app| {
        app.add_plugins(ExportRegistryPlugin::default())
            .add_plugins(WorldInspectorPlugin::default());
        // .add_plugins(EditorPlugin::default());
    });
}
