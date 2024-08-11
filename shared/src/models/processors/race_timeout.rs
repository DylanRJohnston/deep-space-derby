use std::time::{Duration, SystemTime, UNIX_EPOCH};

use im::Vector;

use crate::models::{commands::Command, events::Event, projections};

use super::{Alarm, AlarmProcessor, Processor};

pub struct FinishRace;

impl AlarmProcessor for FinishRace {
    fn alarm(&self, events: &Vector<Event>) -> Option<Alarm> {
        if !matches!(events.last(), Some(Event::RaceStarted { .. })) {
            return None;
        }

        let duration = projections::race_duration(events);

        Some(Alarm(Duration::from_secs_f32(duration)))
    }
}

impl Processor for FinishRace {
    fn process(&self, events: &Vector<Event>) -> Option<Command> {
        let Some(Event::RaceStarted { start }) = events.last() else {
            return None;
        };

        let duration = projections::race_duration(events);

        if SystemTime::now()
            >= UNIX_EPOCH + Duration::from_secs(*start as u64) + Duration::from_secs_f32(duration)
        {
            return Some(Command::FinishRace(()));
        }

        None
    }
}
