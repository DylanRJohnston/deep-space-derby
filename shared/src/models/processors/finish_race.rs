use im::Vector;

use super::{Alarm, AlarmProcessor, Processor};
use crate::models::{commands::Command, events::Event, projections};
use crate::time::*;

pub struct FinishRace;

impl AlarmProcessor for FinishRace {
    fn alarm(&self, events: &Vector<Event>) -> Option<Alarm> {
        if !matches!(events.last(), Some(Event::RaceStarted { .. })) {
            return None;
        }

        let duration = projections::pre_race_duration(events)
            + Duration::from_secs_f32(projections::race_duration(events));

        Some(Alarm(duration))
    }
}

impl Processor for FinishRace {
    fn process(&self, events: &Vector<Event>) -> Option<Command> {
        let Some(Event::RaceStarted { time: start }) = events.last() else {
            return None;
        };

        let duration = projections::pre_race_duration(events)
            + Duration::from_secs_f32(projections::race_duration(events) - 1.);

        if SystemTime::now() >= UNIX_EPOCH + Duration::from_secs(*start as u64) + duration {
            return Some(Command::FinishRace(()));
        }

        None
    }
}
