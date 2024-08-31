use anyhow::{bail, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{
    cards::{Card, Target},
    events::Event,
    projections,
};

use super::{CommandHandler, API};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub card: Card,
    pub target: Target,
}

#[derive(Debug, Copy, Clone)]
pub struct PlayCard;

impl API for PlayCard {
    fn url(game_id: impl std::fmt::Display) -> String {
        format!("/api/object/game/by_code/{}/command/play_card", game_id)
    }
}

impl CommandHandler for PlayCard {
    type Input = Input;

    fn handle(session_id: Uuid, events: &Vector<Event>, input: Input) -> Result<Vec<Event>> {
        if !projections::player_exists(events, session_id) {
            bail!("Player does not exist");
        }

        if !projections::cards_in_hand(events, session_id)
            .into_iter()
            .find(|card| *card == input.card)
            .is_some()
        {
            bail!("Player does not have card in hand");
        }

        if projections::already_played_card_this_round(events, session_id) {
            bail!("Player already played card this round");
        }

        if !projections::valid_target_for_card(events, session_id, input.target.clone()) {
            bail!("Invalid target for card");
        }

        Ok(vec![Event::PlayedCard {
            session_id: session_id,
            card: input.card,
            target: input.target,
        }])
    }
}
