mod player_lobby;
use leptos_reactive::create_memo;
pub use player_lobby::*;

mod host_lobby;
pub use host_lobby::*;

mod game_wrapper;
pub use game_wrapper::*;

mod join_screen;
pub use join_screen::*;

mod main_menu;
pub use main_menu::*;

mod player_pregame;
pub use player_pregame::*;

use leptos::{component, view, IntoView};
use leptos_meta::{provide_meta_context, Stylesheet};
use leptos_router::{Route, Router, Routes};

use crate::{models::events::Event, utils::use_events};

#[component]
pub fn app() -> impl leptos::IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/style.css"/>

        <Router>
            <Routes>
                <Route path="/" view=MainMenu/>
                <Route
                    path="/host/:game_id"
                    view=|| {
                        view! {
                            <GameConnectionWrapper>
                                <GameStateRouter lobby=HostLobby pre_game=PlayerPreGame/>
                            </GameConnectionWrapper>
                        }
                    }
                />

                <Route path="/play" view=JoinScreen/>
                <Route
                    path="/play/:game_id"
                    view=|| {
                        view! {
                            <GameConnectionWrapper>
                                <GameStateRouter lobby=PlayerLobby pre_game=PlayerPreGame/>
                            </GameConnectionWrapper>
                        }
                    }
                />

            </Routes>
        </Router>
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum GameState {
    #[default]
    Lobby,
    PreGame,
    Race,
    Summary,
    FinalScreen,
}

#[component]
pub fn game_state_router<Lobby, LobbyIV, PreGame, PreGameIV>(
    lobby: Lobby,
    pre_game: PreGame,
) -> impl IntoView
where
    Lobby: Fn() -> LobbyIV + 'static,
    LobbyIV: IntoView + 'static,
    PreGame: Fn() -> PreGameIV + 'static,
    PreGameIV: IntoView + 'static,
{
    let events = use_events();

    let state = create_memo(move |_| {
        events()
            .iter()
            .rev()
            .find_map(|event| match event {
                Event::GameCreated { .. } => Some(GameState::Lobby),
                Event::GameStarted => Some(GameState::PreGame),
                Event::RaceStarted => Some(GameState::Race),
                Event::RaceFinished { .. } => Some(GameState::Summary),
                Event::GameFinished => Some(GameState::FinalScreen),
                _ => None,
            })
            .unwrap_or_default()
    });

    move || match state() {
        GameState::Lobby => lobby().into_view(),
        GameState::PreGame => pre_game().into_view(),
        GameState::Race => todo!(),
        GameState::Summary => todo!(),
        GameState::FinalScreen => todo!(),
    }
}

