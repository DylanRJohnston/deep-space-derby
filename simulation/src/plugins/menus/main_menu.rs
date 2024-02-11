#![allow(clippy::type_complexity)]
use bevy::{app::AppExit, prelude::*};

use crate::plugins::pallet;

use super::{components, MenuState};

#[derive(Debug, Reflect, Component)]
pub struct MainMenu;

#[derive(Debug)]
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(MenuState::MainMenu), despawn_main_menu)
            .add_systems(Update, interact_with_join_button)
            .add_systems(Update, interact_with_host_button)
            .add_systems(Update, interact_with_quit_button);
    }
}

fn spawn_main_menu(mut commands: Commands) {
    build_main_menu(&mut commands);
}

fn despawn_main_menu(mut commands: Commands, main_menus: Query<Entity, With<MainMenu>>) {
    for menu in &main_menus {
        commands.entity(menu).despawn_recursive();
    }
}

#[derive(Component, Debug, Reflect)]
struct JoinButton;

#[derive(Component, Debug, Reflect)]
struct HostButton;

#[derive(Component, Debug, Reflect)]
struct QuitButton;

fn button_text(text: &'static str) -> impl FnOnce(&mut ChildBuilder) {
    move |parent| {
        parent.spawn(TextBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font_size: 100.0,
                    ..default()
                },
            ),
            ..default()
        });
    }
}

pub fn build_main_menu(commands: &mut Commands) -> Entity {
    let title = commands
        .spawn(NodeBundle {
            style: Style {
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            background_color: pallet::ORANGE_3.into(),
            ..default()
        })
        .with_children(button_text("Quibble Racing"))
        .id();

    let join_button = commands
        .spawn((JoinButton, components::button()))
        .with_children(button_text("Join"))
        .id();

    let host_button = commands
        .spawn((HostButton, components::button()))
        .with_children(button_text("Host"))
        .id();

    let quit_button = commands
        .spawn((QuitButton, components::button()))
        .with_children(button_text("Quit"))
        .id();

    commands
        .spawn((MainMenu, components::full_screen_container()))
        .push_children(&[title, join_button, host_button, quit_button])
        .id()
}

fn interact_with_join_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<JoinButton>),
    >,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    for (interaction, mut colour) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                colour.0 = pallet::BLUE_5;
                next_state.set(MenuState::JoinMenu);
            }
            Interaction::Hovered => colour.0 = pallet::BLUE_4,
            Interaction::None => colour.0 = pallet::BLUE_3,
        }
    }
}

fn interact_with_host_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<HostButton>),
    >,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    for (interaction, mut colour) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                colour.0 = pallet::BLUE_5;
                next_state.set(MenuState::HostMenu);
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
