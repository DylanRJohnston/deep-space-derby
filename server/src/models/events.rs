use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Event {
    GameCreated { game_id: String },
    PlayerJoined { name: String },
    ChangedProfile,
    GameStarted,
    BoughtCard,
    PlayedCard,
    BorrowedMoney,
    PlacedBet,
    RaceStarted,
    RaceFinished,
    GameFinished,
}
