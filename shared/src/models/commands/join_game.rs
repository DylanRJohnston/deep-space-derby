use std::fmt::Display;

use anyhow::{bail, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::models::{events::Event, game_id::GameID, projections};

use super::{CommandHandler, GameCode, API};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub name: String,
    pub code: GameID,
}

#[derive(Default)]
pub struct JoinGame;

impl GameCode for Input {
    fn game_code(&self) -> GameID {
        self.code
    }
}

impl API for JoinGame {
    fn url(game_id: impl Display) -> String {
        format!("/api/object/game/by_code/{}/command/join_game", game_id)
    }

    fn redirect(game_id: impl Display) -> Option<String> {
        Some(format!("/play/{}", game_id))
    }
}

impl CommandHandler for JoinGame {
    type Input = Input;

    #[instrument(skip_all, fields(input), err)]
    fn handle(session_id: Uuid, events: &Vector<Event>, input: Self::Input) -> Result<Vec<Event>> {
        if events.len() == 0 {
            bail!("cannot join game that doesn't exist");
        }

        if projections::player_exists(events, session_id) {
            return Ok(vec![]);
        }

        if projections::game_has_started(events) {
            bail!("cannot join after game has already started");
        }

        if projections::player_count(events) >= 15 {
            bail!("maximum number of players reached");
        }

        Ok(vec![Event::PlayerJoined {
            name: input.name,
            session_id,
        }])
    }
}
