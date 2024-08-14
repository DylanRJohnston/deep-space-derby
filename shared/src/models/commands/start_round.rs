use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::bail;
use tracing::instrument;
use uuid::Uuid;

use crate::models::{events::Event, projections};

use super::CommandHandler;

pub type Input = ();

pub struct StartRound;

impl CommandHandler for StartRound {
    type Input = ();

    #[instrument(name = "StartRound::handle", err)]
    fn handle(
        session_id: uuid::Uuid,
        events: &im::Vector<Event>,
        input: Self::Input,
    ) -> anyhow::Result<Vec<Event>> {
        if session_id != Uuid::nil() {
            bail!("players may not start the round");
        }

        //TODO: Can server can override??
        if !projections::all_players_ready(events) {
            bail!("round can only start if all players are ready");
        }

        let seed = projections::race_seed_for_round(events, projections::round(events) + 1);
        let monsters = projections::monsters(events, seed);

        Ok(vec![Event::RoundStarted {
            time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32,
            odds: Some(projections::odds(&monsters, seed)),
        }])
    }
}
