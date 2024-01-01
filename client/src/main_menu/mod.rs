#![allow(clippy::type_complexity)]
use bevy::{app::AppExit, prelude::*};

use crate::{pallet, AppState};

#[derive(Debug)]
pub struct MainMenuPlugin;

#[derive(Component, Debug, Reflect)]
struct PlayButton;

#[derive(Component, Debug, Reflect)]
struct QuitButton;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(AppState::MainMenu), despawn_main_menu)
            .add_systems(Update, interact_with_play_button)
            .add_systems(Update, interact_with_quit_button);
    }
}

#[derive(Debug, Reflect, Component)]
pub struct MainMenu;

pub fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    build_main_menu(&mut commands, &asset_server);
}

pub fn despawn_main_menu(mut commands: Commands, main_menus: Query<Entity, With<MainMenu>>) {
    for menu in &main_menus {
        commands.entity(menu).despawn_recursive();
    }
}

pub fn build_main_menu(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Start,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                // background_color: pallet::GREY_4.into(),
                ..default()
            },
            MainMenu,
        ))
        .with_children(|parent| {
            // Title
            parent
                .spawn(NodeBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    background_color: pallet::ORANGE_3.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Quibble Racing",
                            TextStyle {
                                font_size: 100.0,
                                ..default()
                            },
                        ),
                        ..default()
                    });
                });

            // Play
            parent
                .spawn((
                    PlayButton,
                    ButtonBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        background_color: pallet::BLUE_3.into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Play",
                            TextStyle {
                                font_size: 100.0,
                                ..default()
                            },
                        ),
                        ..default()
                    });
                });

            // Quit
            parent
                .spawn((
                    QuitButton,
                    ButtonBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        background_color: pallet::BLUE_3.into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Quit",
                            TextStyle {
                                font_size: 100.0,
                                ..default()
                            },
                        ),
                        ..default()
                    });
                });
        })
        .id()
}

fn interact_with_play_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PlayButton>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut colour) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                colour.0 = pallet::BLUE_5;
                next_state.set(AppState::Playing);
            }
            Interaction::Hovered => colour.0 = pallet::BLUE_4,
            Interaction::None => colour.0 = pallet::BLUE_3,
        }
    }
}

fn interact_with_quit_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<QuitButton>),
    >,
    mut app_exit: EventWriter<AppExit>,
) {
    for (interaction, mut colour) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                colour.0 = pallet::BLUE_5;
                app_exit.send(AppExit);
            }
            Interaction::Hovered => colour.0 = pallet::BLUE_4,
            Interaction::None => colour.0 = pallet::BLUE_3,
        }
    }
}
