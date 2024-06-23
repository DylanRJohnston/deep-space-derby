use leptos::*;
use leptos_router::{Route, Router as LeptosRouter, Routes};

use crate::{
    screens::{game_wrapper::GameConnectionWrapper, host, main_menu::MainMenu, player},
    utils::{send_game_event, use_events},
};
use shared::models::events::Event;

#[component]
pub fn send_events_to_bevy() -> impl IntoView {
    let events = use_events();

    move || {
        events.get().last().map(|event| {
            send_game_event(event.clone());
        })
    }
}

#[component]
pub fn router() -> impl IntoView {
    view! {
        <LeptosRouter>
            <Routes>
                <Route path="/" view=MainMenu/>
                <Route
                    path="/host/:game_id"
                    view=|| {
                        view! {
                            <script type="module">
                                "
                                import init, { sendGameEvent as innerSendGameEvent } from '/pkg/simulation.js';
                                
                                init().catch(() => {
                                    window['innerSendGameEvent'] = innerSendGameEvent;
                                    console.log('Module initialised, flushing pending events')
                                    while (pendingEvents.length > 0) {
                                        let event = pendingEvents.shift();
                                        innerSendGameEvent(event);
                                    }
                                });
                                "
                            </script>
                            <GameConnectionWrapper>
                                <SendEventsToBevy />
                                <GameStateRouter
                                    lobby=host::Lobby
                                    pre_game=host::PreGame
                                    race=host::Race
                                />
                            </GameConnectionWrapper>
                        }
                    }
                />

                <Route path="/play" view=player::Join/>
                <Route
                    path="/play/:game_id"

                    view=|| {
                        view! {
                            <GameConnectionWrapper>
                                <GameStateRouter
                                    lobby=player::Lobby
                                    pre_game=player::PreGame
                                    race=player::Race
                                />
                            </GameConnectionWrapper>
                        }
                    }
                />

            </Routes>
        </LeptosRouter>
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
pub fn game_state_router<Lobby, LobbyIV, PreGame, PreGameIV, Race, RaceIV>(
    lobby: Lobby,
    pre_game: PreGame,
    race: Race,
) -> impl IntoView
where
    Lobby: Fn() -> LobbyIV + 'static,
    LobbyIV: IntoView + 'static,
    PreGame: Fn() -> PreGameIV + 'static,
    PreGameIV: IntoView + 'static,
    Race: Fn() -> RaceIV + 'static,
    RaceIV: IntoView + 'static,
{
    let events = use_events();

    let state = create_memo(move |_| {
        events()
            .iter()
            .rev()
            .find_map(|event| match event {
                Event::GameCreated { .. } => Some(GameState::Lobby),
                Event::GameStarted => Some(GameState::PreGame),
                Event::RaceStarted { .. } => Some(GameState::Race),
                Event::RaceFinished { .. } => Some(GameState::Summary),
                Event::GameFinished => Some(GameState::FinalScreen),
                _ => None,
            })
            .unwrap_or_default()
    });

    move || match state() {
        GameState::Lobby => lobby().into_view(),
        GameState::PreGame => pre_game().into_view(),
        GameState::Race => race().into_view(),
        GameState::Summary => todo!(),
        GameState::FinalScreen => todo!(),
    }
}
