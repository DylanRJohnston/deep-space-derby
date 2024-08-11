use std::time::{SystemTime, UNIX_EPOCH};

use macros::serde_wasm_bindgen;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{game_id::GameID, monsters::RaceResults};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Hash)]
pub struct PlacedBet {
    pub session_id: Uuid,
    pub monster_id: Uuid,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq)]
#[serde_wasm_bindgen]
pub enum Event {
    GameCreated { game_id: GameID },
    PlayerJoined { session_id: Uuid, name: String },
    ChangedProfile { session_id: Uuid, name: String },
    PlayerReady { session_id: Uuid },
    GameStarted { start: u32 },
    BoughtCard { session_id: Uuid },
    PlayedCard,
    BorrowedMoney { session_id: Uuid, amount: u32 },
    PaidBackMoney { session_id: Uuid, amount: u32 },
    PlacedBet(PlacedBet),
    RaceStarted { start: u32 },
    RaceFinished(RaceResults),
    GameFinished,
}

impl Event {
    pub fn start_game_now() -> Event {
        Event::GameStarted {
            start: Event::now(),
        }
    }

    pub fn start_race_now() -> Event {
        Event::RaceStarted {
            start: Event::now(),
        }
    }

    pub fn now() -> u32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
    }
}
