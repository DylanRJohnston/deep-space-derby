use std::fmt::Display;

use super::{Command, Effect};
use crate::models::{events::Event, projections};
use im::Vector;
use uuid::Uuid;

#[derive(Default)]
pub struct ReadyPlayer;

impl Command for ReadyPlayer {
    type Input = ();

    fn url(game_id: impl Display) -> String {
        format!("/api/object/game/by_code/{}/commands/ready_player", game_id)
    }

    fn handle(
        session_id: Uuid,
        events: &Vector<Event>,
        _input: Self::Input,
    ) -> Result<(Vec<Event>, Option<Effect>), String> {
        if !projections::player_exists(events, session_id) {
            return Err("cannot ready a player that doesn't exist".to_owned());
        }

        if projections::game_has_started(events) {
            return Err("cannot ready a player after game has started".to_owned());
        }

        let events = vec![Event::PlayerReady { session_id }];

        let maybe_start_game = |events: &Vector<Event>| {
            projections::all_players_ready(events).then_some(Event::GameStarted)
        };

        Ok((events, Some(Effect::SoftCommand(maybe_start_game))))
    }
}
