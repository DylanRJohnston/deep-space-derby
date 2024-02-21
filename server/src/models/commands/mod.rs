use std::fmt::Display;

use im::Vector;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use super::{events::Event, game_id::GameID};

pub enum Effect {
    Alarm(i64),
    SoftCommand(fn(&Vector<Event>) -> Option<Event>),
}

pub trait Command {
    type Input: Serialize + DeserializeOwned;

    fn url(game_id: impl Display) -> String
    where
        Self: Sized;

    #[allow(unused_variables)]
    fn redirect(game_id: impl Display) -> Option<String>
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
    fn game_code(&self) -> GameID;
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
