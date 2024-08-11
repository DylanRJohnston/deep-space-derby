use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::bail;
use tracing::instrument;
use uuid::Uuid;

use crate::models::{events::Event, projections};

use super::CommandHandler;

pub type Input = ();

pub struct StartGame;

impl CommandHandler for StartGame {
    type Input = ();

    #[instrument(name = "StartGame::handle", err)]
    fn handle(
        session_id: uuid::Uuid,
        events: &im::Vector<Event>,
        input: Self::Input,
    ) -> anyhow::Result<Vec<Event>> {
        if session_id != Uuid::nil() {
            bail!("players may not start the race");
        }

        //TODO: Can server can override??
        if !projections::all_players_ready(events) {
            bail!("game can only start if all players are ready");
        }

        Ok(vec![Event::GameStarted {
            start: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32,
        }])
    }
}
