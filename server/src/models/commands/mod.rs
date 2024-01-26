use im::Vector;
use serde::{de::DeserializeOwned, Serialize};

use super::events::Event;

pub trait Command {
    type Input: Serialize + DeserializeOwned;

    fn url(game_id: &str) -> String;
    fn precondition(events: &Vector<Event>, input: &Self::Input) -> Result<(), String>;
    fn handle(events: &Vector<Event>, input: Self::Input) -> Event;
}

pub mod create_game;
pub use create_game::CreateGame;

pub mod join_game;
pub use join_game::JoinGame;
