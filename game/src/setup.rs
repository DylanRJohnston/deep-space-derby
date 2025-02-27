use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowResolution};
use bevy_tweening::TweeningPlugin;
use iyes_progress::prelude::*;

use crate::plugins::{
    animation_link::AnimationLinkPlugin,
    delayed_command::DelayedCommandPlugin,
    event_stream::EventStreamPlugin,
    monster::MonsterPlugin,
    music::MusicPlugin,
    planets::PlanetsPlugin,
    scenes::{SceneState, ScenesPlugin},
    skinned_mesh::SkinnedMeshPlugin,
    spectators::SpectatorPlugin,
};

#[cfg(target_arch = "wasm32")]
const FILE_PATH: &str = "/assets";

#[cfg(not(target_arch = "wasm32"))]
const FILE_PATH: &str = "assets";

pub fn start(f: impl FnOnce(&mut App)) {
    let mut app = App::new();

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(game_id) = std::env::args().nth(1) {
        let game_id = shared::models::game_code::GameCode::try_from(game_id.as_str())
            .expect("failed to parse game_id");
        app.insert_resource(crate::plugins::event_stream::GameCode(game_id));
    }

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                file_path: FILE_PATH.into(),
                ..Default::default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(1920.0, 1080.0),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(AnimationLinkPlugin)
    .add_plugins(ScenesPlugin)
    .add_plugins(EventStreamPlugin)
    .add_plugins(TweeningPlugin)
    .add_plugins(SpectatorPlugin)
    .add_plugins(PlanetsPlugin)
    .add_plugins(SkinnedMeshPlugin)
    .add_plugins(MusicPlugin)
    .add_plugins(DelayedCommandPlugin)
    .add_plugins(ProgressPlugin::<SceneState>::new().set_asset_tracking(true))
    .add_systems(
        Update,
        ui_progress_bar.run_if(|state: Res<State<SceneState>>| {
            matches!(state.get(), SceneState::Loading | SceneState::Spawning)
        }),
    )
    .add_systems(OnEnter(SceneState::Loading), spawn_progress_bar)
    .add_systems(OnExit(SceneState::Spawning), remove_progress_bar)
    .add_plugins(MonsterPlugin)
    .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
    .insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.0,
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
    let camera = commands.spawn(Camera3d::default()).id();

    commands
        .spawn((
            ProgressBarRoot,
            TargetCamera(camera),
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ProgressMessage,
                Text::new("Loading"),
                TextFont::from_font_size(100.),
            ));

            parent
                .spawn((
                    Node {
                        height: Val::Px(50.0),
                        width: Val::Percent(80.0),
                        ..default()
                    },
                    BackgroundColor(bevy::color::palettes::basic::GRAY.into()),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        ProgressBar,
                        BackgroundColor(bevy::color::palettes::basic::GREEN.into()),
                        Node {
                            height: Val::Px(50.0),
                            width: Val::Percent(0.0),
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
    mut progress_bar: Query<&mut Node, With<ProgressBar>>,
    mut progress_message: Query<&mut Text, With<ProgressMessage>>,
    counter: Option<Res<ProgressTracker<SceneState>>>,
) {
    let progress = counter
        .map(|it| it.get_global_progress().into())
        .unwrap_or(1.0);
    let mut progress_bar = progress_bar.get_single_mut().unwrap();

    progress_bar.width = Val::Percent(100. * progress);

    if progress > 0.99 {
        let mut text = progress_message.get_single_mut().unwrap();

        text.0 = "Creating World".to_string();
    }
}
