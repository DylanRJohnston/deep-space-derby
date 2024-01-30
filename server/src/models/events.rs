use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Event {
    GameCreated { session_id: Uuid, game_id: String },
    PlayerJoined { session_id: Uuid, name: String },
    ChangedProfile { session_id: Uuid, name: String },
    PlayerReady { session_id: Uuid },
    GameStarted,
    BoughtCard,
    PlayedCard,
    BorrowedMoney,
    PlacedBet,
    RaceStarted,
    RaceFinished,
    GameFinished,
}
