use im::Vector;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use super::events::Event;

pub enum Effect {
    Alarm(i64),
    SoftCommand(fn(&Vector<Event>) -> Option<Event>),
}

pub trait Command {
    type Input: Serialize + DeserializeOwned;

    fn url(game_id: &str) -> String
    where
        Self: Sized;

    #[allow(unused_variables)]
    fn redirect(game_id: &str) -> Option<String>
    where
        Self: Sized,
    {
        None
    }

    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        input: Self::Input,
    ) -> Result<(Vec<Event>, Option<Effect>), String>
    where
        Self: Sized;
}

pub trait GameCode {
    fn game_code(&self) -> &str;
}

pub mod create_game;
pub use create_game::CreateGame;

pub mod join_game;
pub use join_game::JoinGame;

pub mod change_profile;
pub use change_profile::ChangeProfile;

pub mod ready_player;
pub use ready_player::ReadyPlayer;

pub mod place_bets;
pub use place_bets::PlaceBets;

