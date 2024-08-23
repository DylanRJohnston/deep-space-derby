use im::Vector;

use super::Processor;
use crate::models::projections;
use crate::models::{commands::Command, events::Event};

pub struct FinishGame;

pub const SUMMARY_DURATION: f32 = 15.0;

impl Processor for FinishGame {
    fn process(&self, events: &Vector<Event>) -> Option<Command> {
        if matches!(events.last(), Some(Event::GameFinished)) {
            return None;
        }

        if projections::game_finished(events) {
            return Some(Command::FinishGame(()));
        }

        None
    }
}
