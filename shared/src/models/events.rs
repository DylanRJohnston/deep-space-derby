use macros::serde_wasm_bindgen;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{game_id::GameID, monsters::Results};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Hash)]
pub struct PlacedBet {
    pub session_id: Uuid,
    pub monster_id: Uuid,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Hash)]
#[serde_wasm_bindgen]
pub enum Event {
    GameCreated { session_id: Uuid, game_id: GameID },
    PlayerJoined { session_id: Uuid, name: String },
    ChangedProfile { session_id: Uuid, name: String },
    PlayerReady { session_id: Uuid },
    GameStarted,
    BoughtCard { session_id: Uuid },
    PlayedCard,
    BorrowedMoney { session_id: Uuid, amount: i32 },
    PaidBackMoney { session_id: Uuid, amount: i32 },
    PlacedBet(PlacedBet),
    RaceStarted,
    RaceFinished(Results),
    GameFinished,
}
