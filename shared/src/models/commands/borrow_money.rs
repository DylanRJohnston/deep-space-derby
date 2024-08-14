use std::fmt::Display;

use anyhow::{bail, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::models::{events::Event, projections};

use super::{CommandHandler, API};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub amount: i32,
}

#[derive(Default)]
pub struct BorrowMoney;

impl API for BorrowMoney {
    fn url(game_id: impl Display) -> String {
        format!("/api/object/game/by_code/{}/commands/borrow_money", game_id)
    }
}

impl CommandHandler for BorrowMoney {
    type Input = Input;

    #[instrument(name = "BorrowMoney::handle", skip(events), err)]
    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        Input { amount }: Self::Input,
    ) -> Result<Vec<Event>> {
        if events.len() == 0 {
            bail!("can't borrow money from a game that doesn't exist");
        }

        if amount == 0 {
            return Ok(vec![]);
        }

        if !projections::player_exists(events, session_id) {
            bail!("only real players can borrow money");
        }

        let debt = projections::debt(events, session_id) as u32;
        let Some(balance) = projections::all_account_balances(events)
            .get(&session_id)
            .copied()
        else {
            bail!("player doesn't exist");
        };

        if amount < 0 && -1 * amount > balance {
            bail!("cannot payback more money than you have");
        }

        let new_debt = debt as i32 + amount;

        if new_debt > 1000 {
            bail!("cannot borrow more than $1000");
        }

        if new_debt < 0 {
            bail!("cannot payback more than you owe");
        }

        match amount {
            0 => Ok(vec![]),
            1.. => Ok(vec![Event::BorrowedMoney {
                session_id,
                amount: amount as u32,
            }]),
            ..0 => Ok(vec![Event::PaidBackMoney {
                session_id: session_id,
                amount: (-1 * amount) as u32,
            }]),
        }
    }
}
