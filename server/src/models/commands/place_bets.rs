use im::Vector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{
    events::{Event, PlacedBet},
    projections,
};

use super::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct Bet {
    pub monster_id: Uuid,
    pub amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub bets: Vec<Bet>,
}

#[derive(Default)]
pub struct PlaceBets;

impl Command for PlaceBets {
    type Input = Input;

    fn url(game_id: &str) -> String {
        format!("/api/object/game/by_code/{}/command/place_bet", game_id)
    }

    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        input: Self::Input,
    ) -> Result<Vec<Event>, String> {
        if !projections::game_has_started(events) {
            return Err("cannot place a bet if the game hasn't started".to_owned());
        }

        if input.bets.iter().any(|it| it.amount < 0) {
            return Err("cannot place a bet with a value less than 0".into());
        }

        let account_balance = projections::account_balance(events)
            .get(&session_id)
            .cloned()
            .unwrap_or_default();

        let total = input.bets.iter().map(|it| it.amount).sum::<i32>();

        if total > account_balance {
            return Err("cannot place a bet with a total value greater than your balance".into());
        }

        Ok(input
            .bets
            .iter()
            .map(|bet| {
                Event::PlacedBet(PlacedBet {
                    session_id,
                    monster_id: bet.monster_id,
                    amount: bet.amount,
                })
            })
            .collect())
    }
}

