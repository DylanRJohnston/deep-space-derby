use leptos::*;
use leptos_router::{Route, Router as LeptosRouter, Routes};
use leptos_use::{use_interval, UseIntervalReturn};

use crate::{
    screens::{game_wrapper::GameConnectionWrapper, host, main_menu::MainMenu, player},
    utils::{send_game_event, use_events, use_session_id},
};
use shared::models::{events::Event, projections};

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
                                import init, { sendGameEvent as innerSendGameEvent } from '/pkg/game.js';
                                
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
                            <GameConnectionWrapper/>
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
                                    wait=player::Wait
                                    summary=player::Summary
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
    Wait,
    Race,
    #[allow(dead_code)]
    Summary,
    FinalScreen,
}

#[component]
pub fn game_state_router<
    Lobby,
    LobbyIV,
    PreGame,
    PreGameIV,
    Race,
    RaceIV,
    Wait,
    WaitIV,
    Summary,
    SummaryIV,
>(
    lobby: Lobby,
    pre_game: PreGame,
    race: Race,
    wait: Wait,
    summary: Summary,
) -> impl IntoView
where
    Lobby: Fn() -> LobbyIV + 'static,
    LobbyIV: IntoView + 'static,
    PreGame: Fn() -> PreGameIV + 'static,
    PreGameIV: IntoView + 'static,
    Race: Fn() -> RaceIV + 'static,
    RaceIV: IntoView + 'static,
    Wait: Fn() -> WaitIV + 'static,
    WaitIV: IntoView + 'static,
    Summary: Fn() -> SummaryIV + 'static,
    SummaryIV: IntoView + 'static,
{
    let events = use_events();
    let player_id = use_session_id();

    let state = create_memo(move |_| {
        let events = events();

        let mut game_state = events
            .iter()
            .rev()
            .find_map(|event| match event {
                Event::GameCreated { .. } => Some(GameState::Lobby),
                Event::RoundStarted { .. } => Some(GameState::PreGame),
                Event::RaceStarted { .. } => Some(GameState::Race),
                Event::RaceFinished { .. } => Some(GameState::Summary),
                Event::GameFinished => Some(GameState::FinalScreen),
                _ => None,
            })
            .unwrap_or_default();

        if GameState::PreGame == game_state && projections::player_has_bet(&events, player_id) {
            game_state = GameState::Wait;
        }

        game_state
    });

    move || match state() {
        GameState::Lobby => lobby().into_view(),
        GameState::PreGame => pre_game().into_view(),
        GameState::Race => race().into_view(),
        GameState::Wait => wait().into_view(),
        GameState::Summary => summary().into_view(),
        GameState::FinalScreen => todo!(),
    }
}
