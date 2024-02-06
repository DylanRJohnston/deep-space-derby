use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct PlacedBet {
    pub session_id: Uuid,
    pub monster_id: Uuid,
    pub amount: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Event {
    GameCreated {
        session_id: Uuid,
        game_id: String,
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
    GameStarted,
    BoughtCard {
        session_id: Uuid,
    },
    PlayedCard,
    BorrowedMoney {
        session_id: Uuid,
        amount: i32,
    },
    PaidBackMoney {
        session_id: Uuid,
        amount: i32,
    },
    PlacedBet(PlacedBet),
    RaceStarted,
    RaceFinished {
        first: Uuid,
        second: Uuid,
        third: Uuid,
    },
    GameFinished,
}

