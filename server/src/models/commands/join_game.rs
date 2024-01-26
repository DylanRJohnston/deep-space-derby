use im::Vector;
use serde::{Deserialize, Serialize};

use crate::models::{events::Event, projections};

use super::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub name: String,
}

pub struct JoinGame;

impl Command for JoinGame {
    type Input = Input;

    fn url(game_id: &str) -> String {
        format!("/api/object/game/by_code/{}/command/join_game", game_id)
    }

    fn precondition(events: &Vector<Event>, _input: &Self::Input) -> Result<(), String> {
        if projections::game_has_started(events) {
            return Err("cannot join after game has already started".to_owned());
        }

        if projections::player_count(events) >= 15 {
            return Err("maximum number of players reached".to_owned());
        }

        Ok(())
    }

    fn handle(_events: &Vector<Event>, input: Self::Input) -> Event {
        Event::PlayerJoined { name: input.name }
    }
}
