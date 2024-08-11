use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::bail;
use tracing::instrument;
use uuid::Uuid;

use crate::models::events::Event;

use super::CommandHandler;

pub type Input = ();

pub struct StartRace;

impl CommandHandler for StartRace {
    type Input = ();

    #[instrument(name = "StartRace::handle", err)]
    fn handle(
        session_id: uuid::Uuid,
        events: &im::Vector<Event>,
        input: Self::Input,
    ) -> anyhow::Result<Vec<Event>> {
        if session_id != Uuid::nil() {
            bail!("players may not start the race");
        }

        //TODO: can we enforce this condition if the server needs to trigger a timeout?
        // if !projections::all_players_have_bet(events) {
        //     bail!("race can only start if all players have bet");
        // }

        Ok(vec![Event::RaceStarted {
            start: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32,
        }])
    }
}
