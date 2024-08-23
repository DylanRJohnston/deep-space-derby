use anyhow::bail;
use tracing::instrument;
use uuid::Uuid;

use crate::models::{events::Event, projections};

use super::CommandHandler;

pub type Input = ();

pub struct FinishGame;

impl CommandHandler for FinishGame {
    type Input = ();

    #[instrument(skip(events), err)]
    fn handle(
        session_id: uuid::Uuid,
        events: &im::Vector<Event>,
        input: Self::Input,
    ) -> anyhow::Result<Vec<Event>> {
        if session_id != Uuid::nil() {
            bail!("players may not finish the game");
        }

        if !projections::game_finished(events) {
            bail!("game is not finished");
        }

        if matches!(events.last(), Some(Event::GameFinished)) {
            return Ok(vec![]);
        }

        Ok(vec![Event::GameFinished])
    }
}
