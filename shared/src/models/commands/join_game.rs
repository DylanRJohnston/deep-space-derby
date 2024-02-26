use std::fmt::Display;

use im::Vector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{events::Event, game_id::GameID, projections};

use super::{Command, Effect, GameCode};

#[derive(Debug, Serialize, Deserialize)]
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

impl Command for JoinGame {
    type Input = Input;

    fn url(game_id: impl Display) -> String {
        format!("/api/object/game/by_code/{}/command/join_game", game_id)
    }

    fn redirect(game_id: impl Display) -> Option<String> {
        Some(format!("/play/{}", game_id))
    }

    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        input: Self::Input,
    ) -> Result<(Vec<Event>, Option<Effect>), String> {
        if projections::player_exists(events, session_id) {
            return Ok((vec![], None));
        }

        if projections::game_has_started(events) {
            return Err("cannot join after game has already started".to_owned());
        }

        if projections::player_count(events) >= 15 {
            return Err("maximum number of players reached".to_owned());
        }

        Ok((
            vec![Event::PlayerJoined {
                name: input.name,
                session_id,
            }],
            None,
        ))
    }
}
