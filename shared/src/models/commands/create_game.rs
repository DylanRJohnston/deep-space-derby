use std::fmt::Display;

use anyhow::{bail, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::models::{events::Event, game_id::GameID};

use super::{Command, Effect, GameCode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub code: GameID,
}

impl GameCode for Input {
    fn game_code(&self) -> GameID {
        self.code
    }
}

#[derive(Default)]
pub struct CreateGame;

impl Command for CreateGame {
    type Input = Input;

    fn url(game_id: impl Display) -> String {
        format!("/api/object/game/by_code/{}/commands/create_game", game_id)
    }

    fn redirect(game_id: impl Display) -> Option<String> {
        Some(format!("/host/{}", game_id))
    }

    #[instrument(name = "CreateGame::handle", err)]
    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        input: Self::Input,
    ) -> Result<(Vec<Event>, Option<Effect>)> {
        if !events.is_empty() {
            bail!("create game cannot be called after the game has already been created");
        }

        Ok((
            vec![Event::GameCreated {
                game_id: input.code,
                // session_id,
            }],
            None,
        ))
    }
}
