use im::Vector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::events::Event;

use super::{Command, Effect, GameCode};

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub code: String,
}

impl GameCode for Input {
    fn game_code(&self) -> &str {
        &self.code
    }
}

#[derive(Default)]
pub struct CreateGame;

impl Command for CreateGame {
    type Input = Input;

    fn url(game_id: &str) -> String {
        format!("/api/object/game/by_code/{}/commands/create_game", game_id)
    }

    fn redirect(game_id: &str) -> Option<String> {
        Some(format!("/host/{}", game_id))
    }

    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        input: Self::Input,
    ) -> Result<(Vec<Event>, Option<Effect>), String> {
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

        Ok((
            vec![Event::GameCreated {
                game_id: input.code,
                session_id,
            }],
            None,
        ))
    }
}

