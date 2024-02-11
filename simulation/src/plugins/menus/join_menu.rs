#![allow(clippy::type_complexity)]
use bevy::prelude::*;

use crate::plugins::{menus::components, pallet};

use super::MenuState;

#[derive(Debug)]
pub struct JoinMenuPlugin;

impl Plugin for JoinMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::JoinMenu), spawn_join_menu)
            .add_systems(OnExit(MenuState::JoinMenu), despawn_join_menu)
            .add_systems(
                Update,
                interact_with_join_button.run_if(in_state(MenuState::JoinMenu)),
            )
            .add_systems(
                Update,
                interact_with_back_button.run_if(in_state(MenuState::JoinMenu)),
            )
            .add_systems(
                Update,
                handle_text_input.run_if(in_state(MenuState::JoinMenu)),
            );
    }
}

#[derive(Debug, Reflect, Component)]
struct JoinMenu;

#[derive(Debug, Reflect, Component)]
struct BackButton;

#[derive(Debug, Reflect, Component)]
struct JoinButton;

#[derive(Debug, Reflect, Component)]
struct LobbyCode;

#[derive(Debug, Reflect, Component)]
struct Disabled;

fn spawn_join_menu(mut commands: Commands) {
    let join_lobby = commands
        .spawn(TextBundle {
            text: Text::from_section(
                "Join Lobby",
                TextStyle {
                    font_size: 100.0,
                    ..Default::default()
                },
            ),
            ..default()
        })
        .id();

    let code_entry = {
        let code_text = commands
            .spawn(TextBundle {
                text: Text::from_section(
                    "Code:",
                    TextStyle {
                        font_size: 80.0,
                        ..default()
                    },
                ),
                ..default()
            })
            .id();

        let code_input = {
            let code_input_text = commands
                .spawn((
                    LobbyCode,
                    TextBundle {
                        text: Text::from_section(
                            "",
                            TextStyle {
                                font_size: 80.0,
                                ..default()
                            },
                        ),
                        ..default()
                    },
                ))
                .id();

            commands
                .spawn(NodeBundle {
                    style: Style {
                        min_width: Val::Px(200.0),
                        min_height: Val::Px(80.0),
                        border: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    border_color: Color::WHITE.into(),
                    ..default()
                })
                .push_children(&[code_input_text])
                .id()
        };

        commands
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                },
                ..default()
            })
            .push_children(&[code_text, code_input])
            .id()
    };

    let button_tray = {
        let button_tray_text_style = TextStyle {
            font_size: 100.0,
            ..default()
        };

        let button_bundle = components::button();

        let back_button = {
            let back_text = commands
                .spawn(TextBundle {
                    text: Text::from_section("Back", button_tray_text_style.clone()),
                    ..default()
                })
                .id();

            commands
                .spawn((BackButton, button_bundle.clone()))
                .push_children(&[back_text])
                .id()
        };

        let join_button = {
            let join_text = commands
                .spawn(TextBundle {
                    text: Text::from_section("Join", button_tray_text_style.clone()),
                    ..default()
                })
                .id();

            let mut button_bundle = button_bundle.clone();
            button_bundle.background_color = pallet::GREY_1.into();

            commands
                .spawn((JoinButton, Disabled, button_bundle))
                .push_children(&[join_text])
                .id()
        };

        commands
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            })
            .push_children(&[back_button, join_button])
            .id()
    };

    commands
        .spawn((
            JoinMenu,
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .push_children(&[join_lobby, code_entry, button_tray]);
}

fn despawn_join_menu(mut commands: Commands, join_menus: Query<Entity, With<JoinMenu>>) {
    for join_menu in &join_menus {
        commands.entity(join_menu).despawn_recursive();
    }
}

fn interact_with_join_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<JoinButton>, Without<Disabled>),
    >,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    for (interaction, mut colour) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                colour.0 = pallet::BLUE_5;
                next_state.set(MenuState::Joining);
            }
            Interaction::Hovered => colour.0 = pallet::BLUE_4,
            Interaction::None => colour.0 = pallet::BLUE_3,
        }
    }
}

fn interact_with_back_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<BackButton>, Without<Disabled>),
    >,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    for (interaction, mut colour) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                colour.0 = pallet::BLUE_5;
                next_state.set(MenuState::MainMenu);
            }
            Interaction::Hovered => colour.0 = pallet::BLUE_4,
            Interaction::None => colour.0 = pallet::BLUE_3,
        }
    }
}

fn handle_text_input(
    mut evr_char: EventReader<ReceivedCharacter>,
    kbd: Res<Input<KeyCode>>,
    mut join_button: Query<(Entity, &mut BackgroundColor), With<JoinButton>>,
    mut lobby_code: Query<&mut Text, With<LobbyCode>>,
    mut commands: Commands,
) {
    let text = &mut lobby_code.get_single_mut().unwrap().sections[0].value;

    if kbd.just_pressed(KeyCode::Back) {
        text.pop();
    }

    for ev in evr_char.read() {
        if ev.char.is_control() {
            continue;
        }

        if text.len() >= 4 {
            continue;
        }

        text.push(ev.char.to_ascii_uppercase());

        for (entity, mut background_colour) in &mut join_button {
            if text.len() < 4 {
                background_colour.0 = Color::BLACK;

                commands.entity(entity).insert(Disabled);
            } else {
                background_colour.0 = pallet::BLUE_3;
                commands.entity(entity).remove::<Disabled>();
            }
        }
    }
}
