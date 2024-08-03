use std::fmt::Display;

use anyhow::{bail, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::models::{events::Event, projections};

use super::{Command, Effect};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[instrument(skip_all, fields(input), err)]
    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        input: Self::Input,
    ) -> Result<(Vec<Event>, Option<Effect>)> {
        if !projections::player_exists(events, session_id) {
            bail!("cannot modify player that doesn't exist");
        }

        if projections::game_has_started(events) {
            bail!("cannot modify profile after game has started");
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
