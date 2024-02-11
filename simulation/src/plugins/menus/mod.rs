use bevy::prelude::*;

pub mod components;
pub mod join_menu;
pub mod main_menu;

#[derive(Debug)]
pub struct MenuPlugin;

#[derive(Debug, States, Hash, Eq, PartialEq, Clone, Default)]
pub enum MenuState {
    #[default]
    MainMenu,
    JoinMenu,
    Joining,
    HostMenu,
    // None,
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MenuState>()
            .add_plugins(main_menu::MainMenuPlugin)
            .add_plugins(join_menu::JoinMenuPlugin);
    }
}
