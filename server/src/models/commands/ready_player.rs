use super::Command;
use crate::models::{events::Event, projections};
use im::Vector;
use uuid::Uuid;

#[derive(Default)]
pub struct ReadyPlayer;

impl Command for ReadyPlayer {
    type Input = ();

    fn url(game_id: &str) -> String {
        format!("/api/object/game/by_code/{}/commands/ready_player", game_id)
    }

    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        _input: Self::Input,
    ) -> Result<Vec<Event>, String> {
        if !projections::player_exists(events, session_id) {
            return Err("cannot ready a player that doesn't exist".to_owned());
        }

        if projections::game_has_started(events) {
            return Err("cannot ready a player after game has started".to_owned());
        }

        let event = Event::PlayerReady { session_id };
        let mut new_events = vec![event.clone()];

        if projections::all_players_ready(&{
            let mut events = events.clone();
            events.push_back(event);

            events
        }) {
            new_events.push(Event::GameStarted)
        }

        Ok(new_events)
    }
}
