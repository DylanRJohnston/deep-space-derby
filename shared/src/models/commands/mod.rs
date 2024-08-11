use std::fmt::Display;

use im::Vector;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use super::{events::Event, game_id::GameID};

#[derive(Debug)]
pub enum Effect {
    Alarm(i64),
    SoftCommand(fn(&Vector<Event>) -> Option<Event>),
}

pub trait CommandHandler {
    type Input: Serialize + DeserializeOwned + std::fmt::Debug + Send + Sync + 'static;

    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        input: Self::Input,
    ) -> anyhow::Result<Vec<Event>>;
}

pub trait API {
    fn url(game_id: impl Display) -> String;

    #[allow(unused_variables)]
    fn redirect(game_id: impl Display) -> Option<String> {
        None
    }
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

pub mod start_game;
pub use start_game::StartGame;

pub mod place_bets;
pub use place_bets::PlaceBets;

pub mod start_race;
pub use start_race::StartRace;

pub mod finish_race;
pub use finish_race::FinishRace;

pub mod borrow_money;
pub use borrow_money::BorrowMoney;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Command {
    CreateGame(create_game::Input),
    JoinGame(join_game::Input),
    ChangeProfile(change_profile::Input),
    ReadyPlayer(ready_player::Input),
    StartGame(start_game::Input),
    StartRace(start_race::Input),
    PlaceBets(place_bets::Input),
    FinishRace(finish_race::Input),
}

impl CommandHandler for Command {
    type Input = Self;

    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        input: Self::Input,
    ) -> anyhow::Result<Vec<Event>> {
        match input {
            Command::CreateGame(input) => CreateGame::handle(session_id, events, input),
            Command::JoinGame(input) => JoinGame::handle(session_id, events, input),
            Command::ChangeProfile(input) => ChangeProfile::handle(session_id, events, input),
            Command::ReadyPlayer(input) => ReadyPlayer::handle(session_id, events, input),
            Command::StartGame(input) => StartGame::handle(session_id, events, input),
            Command::StartRace(input) => StartRace::handle(session_id, events, input),
            Command::PlaceBets(input) => PlaceBets::handle(session_id, events, input),
            Command::FinishRace(input) => FinishRace::handle(session_id, events, input),
        }
    }
}
