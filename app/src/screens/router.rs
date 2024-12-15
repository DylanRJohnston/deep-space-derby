use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router as LeptosRouter, Routes},
    ParamSegment, StaticSegment,
};

use crate::{
    screens::{game_wrapper::GameConnectionWrapper, host, main_menu::MainMenu, player},
    utils::{send_game_event, use_events, use_session_id},
};
use shared::models::{
    events::{Event, EventStream},
    projections,
};

#[component]
pub fn send_events_to_bevy() -> impl IntoView {
    let events = use_events();

    Effect::new(move |_| {
        let events = events();

        // This is really inefficient sending the entire event
        // But I'm about to remove comms between the UI and the game
        send_game_event(EventStream::Events(Vec::from_iter(events.into_iter())));
    });
}

#[component]
pub fn router() -> impl IntoView {
    view! {
        <LeptosRouter>
            <Routes fallback=move || ()>
                <Route path=StaticSegment("") view=MainMenu/>
                <Route
                    path=(StaticSegment("host"), ParamSegment("game_id"))
                    view=|| {
                        view! {
                            <script type="module">
                                "
                                import init, { sendGameEvent as innerSendGameEvent } from '/pkg/game.js';
                                
                                init().catch(() => {
                                    globalThis['innerSendGameEvent'] = innerSendGameEvent;
                                    console.log('Module initialised, flushing pending events')
                                    console.log(pendingEvents);
                                    while (pendingEvents.length > 0) {
                                        let event = pendingEvents.shift();
                                        innerSendGameEvent(event);
                                    }
                                });
                                "
                            </script>
                            <GameConnectionWrapper>
                                <SendEventsToBevy/>
                                <GameStateRouter
                                    lobby=host::Lobby
                                    pre_game=host::PreGame
                                    race=host::Race
                                    wait=|| {}
                                    summary=host::Results
                                />
                            </GameConnectionWrapper>
                        }
                    }
                />

                <Route path=StaticSegment("play") view=player::Join/>
                <Route
                    path=(StaticSegment("play"), ParamSegment("game_id"))
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
    Lobby: Fn() -> LobbyIV + Send + 'static,
    LobbyIV: IntoView + 'static,
    PreGame: Fn() -> PreGameIV + Send + 'static,
    PreGameIV: IntoView + 'static,
    Race: Fn() -> RaceIV + Send + 'static,
    RaceIV: IntoView + 'static,
    Wait: Fn() -> WaitIV + Send + 'static,
    WaitIV: IntoView + 'static,
    Summary: Fn() -> SummaryIV + Send + 'static,
    SummaryIV: IntoView + 'static,
{
    let events = use_events();
    let player_id = use_session_id();

    let state = Memo::new(move |_| {
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
        GameState::Lobby => lobby().into_any(),
        GameState::PreGame => pre_game().into_any(),
        GameState::Race => race().into_any(),
        GameState::Wait => wait().into_any(),
        GameState::Summary | GameState::FinalScreen => summary().into_any(),
    }
}
