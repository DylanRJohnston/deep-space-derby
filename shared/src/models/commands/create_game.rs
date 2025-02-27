use std::fmt::Display;

use anyhow::{bail, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::models::events::{Event, Settings};

use super::{CommandHandler, GameCode, HasGameCode, API};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub code: GameCode,
    pub settings: Option<Settings>,
}

impl HasGameCode for Input {
    fn game_code(&self) -> GameCode {
        self.code
    }
}

#[derive(Default)]
pub struct CreateGame;

impl API for CreateGame {
    fn url(game_id: impl Display) -> String {
        format!("/api/object/game/by_code/{}/commands/create_game", game_id)
    }

    fn redirect(game_id: impl Display) -> Option<String> {
        Some(format!("/host/{}", game_id))
    }
}

impl CommandHandler for CreateGame {
    type Input = Input;

    #[instrument(name = "CreateGame::handle", err)]
    fn handle(session_id: Uuid, events: &Vector<Event>, input: Self::Input) -> Result<Vec<Event>> {
        if !events.is_empty() {
            bail!("create game cannot be called after the game has already been created");
        }

        Ok(vec![Event::GameCreated {
            game_id: input.code,
            settings: input.settings.unwrap_or_default(),
        }])
    }
}
