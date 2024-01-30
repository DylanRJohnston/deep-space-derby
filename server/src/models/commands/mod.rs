use im::Vector;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use super::events::Event;

pub trait Command {
    type Input: Serialize + DeserializeOwned;

    fn url(game_id: &str) -> String;
    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        input: Self::Input,
    ) -> Result<Option<Event>, String>;
}

pub mod create_game;
pub use create_game::CreateGame;

pub mod join_game;
pub use join_game::JoinGame;

pub mod change_profile;
pub use change_profile::ChangeProfile;

pub mod ready_player;
pub use ready_player::ReadyPlayer;
