use std::fmt::Display;

use im::Vector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{events::Event, projections};

use super::{Command, Effect};

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub name: String,
}

#[derive(Default)]
pub struct ChangeProfile;

impl Command for ChangeProfile {
    type Input = Input;

    fn url(game_id: impl Display) -> String {
        format!(
            "/api/object/game/by_code/{}/commands/change_profile",
            game_id
        )
    }

    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        input: Self::Input,
    ) -> Result<(Vec<Event>, Option<Effect>), String> {
        if !projections::player_exists(events, session_id) {
            return Err("cannot modify player that doesn't exist".to_owned());
        }

        if projections::game_has_started(events) {
            return Err("cannot modify profile after game has started".to_owned());
        }

        Ok((
            vec![Event::ChangedProfile {
                session_id,
                name: input.name,
            }],
            None,
        ))
    }
}
