use anyhow::{bail, Result};
use im::Vector;
use uuid::Uuid;

use crate::models::{events::Event, projections};

use super::{CommandHandler, API};

pub type Input = ();

#[derive(Debug, Copy, Clone)]
pub struct BuyCard;

impl API for BuyCard {
    fn url(game_id: impl std::fmt::Display) -> String {
        format!("/api/object/game/by_code/{}/command/buy_card", game_id)
    }
}

impl CommandHandler for BuyCard {
    type Input = Input;

    fn handle(session_id: Uuid, events: &Vector<Event>, _input: Input) -> Result<Vec<Event>> {
        if !projections::player_exists(events, session_id) {
            bail!("Player does not exist");
        }

        if projections::cards_in_hand(events, session_id).len() >= 5 {
            bail!("Player already has 5 cards in hand");
        }

        if projections::account_balance(events, session_id) < 100 {
            bail!("Player does not have enough money");
        }

        Ok(vec![Event::BoughtCard {
            session_id: session_id,
            card: projections::draw_card_from_deck(events),
        }])
    }
}
