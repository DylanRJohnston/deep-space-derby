use im::Vector;
use serde::{Deserialize, Serialize};

use crate::models::events::Event;

use super::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub code: String,
}

pub struct CreateGame;

impl Command for CreateGame {
    type Input = Input;

    fn url(game_id: &str) -> String {
        format!("/api/object/game/by_code/{}/commands/create_game", game_id)
    }

    fn precondition(events: &Vector<Event>, input: &Self::Input) -> Result<(), String> {
        if !events.is_empty() {
            return Err(
                "create game cannot be called after the game has already been created".to_owned(),
            );
        }

        if input.code.len() != 6 {
            return Err(format!(
                "game code must be exactly 6, got {}",
                input.code.len()
            ));
        }

        Ok(())
    }

    fn handle(_events: &Vector<Event>, input: Self::Input) -> Event {
        Event::GameCreated {
            game_id: input.code,
        }
    }
}
