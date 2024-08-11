use std::fmt::Display;

use super::{CommandHandler, API};
use crate::models::{events::Event, projections};
use anyhow::{bail, Result};
use im::Vector;
use uuid::Uuid;

pub type Input = ();

#[derive(Default)]
pub struct ReadyPlayer;

impl API for ReadyPlayer {
    fn url(game_id: impl Display) -> String {
        format!("/api/object/game/by_code/{}/commands/ready_player", game_id)
    }
}

impl CommandHandler for ReadyPlayer {
    type Input = ();

    fn handle(session_id: Uuid, events: &Vector<Event>, _input: Self::Input) -> Result<Vec<Event>> {
        if !projections::player_exists(events, session_id) {
            bail!("cannot ready a player that doesn't exist");
        }

        if projections::game_has_started(events) {
            bail!("cannot ready a player after game has started");
        }

        let events = vec![Event::PlayerReady { session_id }];

        Ok(events)
    }
}
