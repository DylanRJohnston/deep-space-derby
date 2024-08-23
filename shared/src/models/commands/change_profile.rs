use std::fmt::Display;

use anyhow::{bail, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::models::{
    events::Event,
    projections::{self, PlayerInfo},
};

use super::{CommandHandler, API};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub name: String,
}

#[derive(Default)]
pub struct ChangeProfile;

impl API for ChangeProfile {
    fn url(game_id: impl Display) -> String {
        format!(
            "/api/object/game/by_code/{}/commands/change_profile",
            game_id
        )
    }
}

impl CommandHandler for ChangeProfile {
    type Input = Input;

    #[instrument(skip_all, fields(input), err)]
    fn handle(session_id: Uuid, events: &Vector<Event>, input: Self::Input) -> Result<Vec<Event>> {
        if !projections::player_exists(events, session_id) {
            bail!("cannot modify player that doesn't exist");
        }

        if projections::game_has_started(events) {
            bail!("cannot modify profile after game has started");
        }

        match projections::player_info(events, session_id) {
            Some(PlayerInfo { name, .. }) if name == input.name => Ok(vec![]),
            _ => Ok(vec![Event::ChangedProfile {
                session_id,
                name: input.name,
            }]),
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::anyhow;
    use im::vector;
    use uuid::Uuid;

    use crate::models::{
        commands::{change_profile, ChangeProfile, CommandHandler},
        events::Event,
        game_id::GameID,
    };

    #[test]
    fn cannot_modify_player_that_doesnt_exist() -> anyhow::Result<()> {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameID::random()
            },
            Event::PlayerJoined {
                session_id: a,
                name: "A".into()
            },
        ];

        assert_eq!(
            "cannot modify player that doesn't exist",
            ChangeProfile::handle(b, &events, change_profile::Input { name: "B".into() })
                .err()
                .ok_or_else(|| anyhow!("failed to fail"))?
                .root_cause()
                .to_string(),
        );

        Ok(())
    }
}
