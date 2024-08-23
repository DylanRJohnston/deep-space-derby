use std::ops::Deref;

use macros::serde_wasm_bindgen;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    cards::{Card, Target},
    game_id::GameID,
    projections::RaceResults,
};

use crate::time::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Hash)]
pub struct PlacedBet {
    pub session_id: Uuid,
    pub monster_id: Uuid,
    pub amount: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct Odds(pub [(Uuid, f32); 3]);

impl Deref for Odds {
    type Target = [(Uuid, f32); 3];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait OddsExt {
    fn odds(&self, monster_id: Uuid) -> f32;
    fn payout(&self, monster_id: Uuid) -> f32;
}

const PAYOUT_MAX: f32 = 10.0;

impl OddsExt for Odds {
    fn odds(&self, monster_id: Uuid) -> f32 {
        self.0
            .iter()
            .find(|(id, _)| *id == monster_id)
            .map(|(_, odds)| odds)
            .copied()
            .unwrap_or_else(|| {
                tracing::warn!(?monster_id, "no odds found for monster");
                1. / 3.0
            })
    }

    fn payout(&self, monster_id: Uuid) -> f32 {
        f32::min(1.0 / self.odds(monster_id), PAYOUT_MAX)
    }
}

impl OddsExt for Option<Odds> {
    fn odds(&self, monster_id: Uuid) -> f32 {
        self.map(|inner| inner.odds(monster_id)).unwrap_or_else(|| {
            tracing::warn!("getting default odds");
            1. / 3.
        })
    }

    fn payout(&self, monster_id: Uuid) -> f32 {
        self.map(|inner| inner.payout(monster_id))
            .unwrap_or_else(|| {
                tracing::warn!("getting default payout");
                3.0
            })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[serde_wasm_bindgen]
pub enum Event {
    GameCreated {
        game_id: GameID,
    },
    PlayerJoined {
        session_id: Uuid,
        name: String,
    },
    ChangedProfile {
        session_id: Uuid,
        name: String,
    },
    PlayerReady {
        session_id: Uuid,
    },
    RoundStarted {
        time: u32,
        odds: Option<Odds>,
    },
    BoughtCard {
        session_id: Uuid,
        card: Card,
    },
    PlayedCard {
        session_id: Uuid,
        card: Card,
        target: Target,
    },
    BorrowedMoney {
        session_id: Uuid,
        amount: u32,
    },
    PaidBackMoney {
        session_id: Uuid,
        amount: u32,
    },
    PlacedBet(PlacedBet),
    RaceStarted {
        time: u32,
    },
    RaceFinished {
        time: u32,
        results: RaceResults,
    },
    GameFinished,
}

impl Event {
    pub fn start_round_now() -> Event {
        Event::RoundStarted {
            time: Event::now(),
            odds: None,
        }
    }

    pub fn start_race_now() -> Event {
        Event::RaceStarted { time: Event::now() }
    }

    pub fn now() -> u32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
    }
}
