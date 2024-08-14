use std::fmt::Display;

use anyhow::{bail, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::models::{
    events::{Event, PlacedBet},
    projections::{self},
};

use super::{CommandHandler, API};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bet {
    pub monster_id: Uuid,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub bets: Vec<Bet>,
}

#[derive(Default)]
pub struct PlaceBets;

impl API for PlaceBets {
    fn url(game_id: impl Display) -> String {
        format!("/api/object/game/by_code/{}/command/place_bet", game_id)
    }
}

impl CommandHandler for PlaceBets {
    type Input = Input;

    #[instrument(skip(events), err)]
    fn handle(session_id: Uuid, events: &Vector<Event>, input: Self::Input) -> Result<Vec<Event>> {
        if !projections::game_has_started(events) {
            bail!("cannot place a bet if the game hasn't started");
        }

        if input.bets.iter().any(|it| it.amount < 0) {
            bail!("cannot place a bet with a value less than 0");
        }

        let account_balance = projections::all_account_balances(events)
            .get(&session_id)
            .cloned()
            .unwrap_or_default();

        let total = input.bets.iter().map(|it| it.amount).sum::<i32>();

        if total > account_balance {
            bail!("cannot place a bet with a total value greater than your balance");
        }

        // if total < projections::minimum_bet(events) {
        //     bail!("bet cannot be less than the minimum bet");
        // }

        let race_seed = projections::race_seed(events);
        let monsters = projections::monsters(race_seed);

        for bet in input.bets.iter() {
            if !monsters
                .iter()
                .any(|monster| bet.monster_id == monster.uuid)
            {
                bail!("failed to find monster corresponding to bet");
            }
        }

        let events = input
            .bets
            .iter()
            .filter(|bet| bet.amount > 0)
            .map(|bet| {
                Event::PlacedBet(PlacedBet {
                    session_id,
                    monster_id: bet.monster_id,
                    amount: bet.amount,
                })
            })
            .collect();

        Ok(events)
    }
}
