use std::process::Command;

use bevy::prelude::*;
use bevy_gltf_components::GltfComponentsSet;
use bevy_tweening::TweeningPlugin;
use iyes_progress::{ProgressCounter, ProgressPlugin, TrackedProgressSet};

use crate::plugins::{
    event_stream::EventStreamPlugin, monster::MonsterPlugin, planets::PlanetsPlugin,
    scenes::SceneState, scenes::ScenesPlugin, spectators::SpectatorPlugin,
};

pub fn start(f: impl FnOnce(&mut App)) {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window { ..default() }),
                ..default()
            })
            .set(AssetPlugin {
                #[cfg(target_arch = "wasm32")]
                file_path: "/assets".into(),

                #[cfg(not(target_arch = "wasm32"))]
                file_path: "assets".into(),

                ..Default::default()
            }),
    )
    .add_plugins(bevy_gltf_blueprints::BlueprintsPlugin {
        library_folder: "library".into(),
        material_library: false,
        legacy_mode: false,
        ..Default::default()
    })
    .add_plugins(ScenesPlugin)
    .add_plugins(EventStreamPlugin)
    .add_plugins(TweeningPlugin)
    .add_plugins(SpectatorPlugin)
    .add_plugins(PlanetsPlugin)
    .add_plugins(
        ProgressPlugin::new(SceneState::Loading)
            .continue_to(SceneState::Spawning)
            .track_assets(),
    )
    .add_systems(
        Update,
        ui_progress_bar
            .after(TrackedProgressSet)
            .run_if(in_state(SceneState::Loading)),
    )
    .add_systems(OnEnter(SceneState::Loading), spawn_progress_bar)
    .add_systems(OnEnter(SceneState::Lobby), remove_progress_bar)
    .add_plugins(MonsterPlugin)
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.075,
    });

    f(&mut app);

    app.run();
}

#[derive(Component)]
pub struct ProgressBarRoot;

#[derive(Component)]
pub struct ProgressBar;

#[derive(Component)]
pub struct ProgressMessage;

fn spawn_progress_bar(mut commands: Commands) {
    let camera = commands.spawn(Camera3dBundle::default()).id();

    commands
        .spawn((
            ProgressBarRoot,
            TargetCamera(camera),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ProgressMessage,
                TextBundle {
                    text: Text::from_section(
                        "Loading",
                        TextStyle {
                            font_size: 100.0,
                            ..default()
                        },
                    ),
                    ..default()
                },
            ));

            parent
                .spawn(NodeBundle {
                    background_color: Color::GRAY.into(),
                    style: Style {
                        height: Val::Px(50.0),
                        width: Val::Percent(80.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        ProgressBar,
                        NodeBundle {
                            background_color: Color::GREEN.into(),
                            style: Style {
                                height: Val::Px(50.0),
                                width: Val::Percent(0.0),
                                ..default()
                            },
                            ..default()
                        },
                    ));
                });
        });
}

fn remove_progress_bar(progress_bar: Query<Entity, With<ProgressBarRoot>>, mut commands: Commands) {
    let progress_bar = progress_bar.get_single().unwrap();

    commands.entity(progress_bar).despawn_recursive();
}

fn ui_progress_bar(
    mut progress_bar: Query<&mut Style, With<ProgressBar>>,
    mut progress_message: Query<&mut Text, With<ProgressMessage>>,
    counter: Res<ProgressCounter>,
) {
    let progress = counter.progress();
    let mut progress_bar = progress_bar.get_single_mut().unwrap();

    let progress = Into::<f32>::into(progress);

    progress_bar.width = Val::Percent(100. * progress);

    if progress > 0.99 {
        let mut text = progress_message.get_single_mut().unwrap();

        "Creating World".clone_into(&mut text.sections[0].value);
    }
}
