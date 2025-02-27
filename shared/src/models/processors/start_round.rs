use im::Vector;

use super::{Alarm, AlarmProcessor, Processor};
use crate::models::projections;
use crate::models::{commands::Command, events::Event};
use crate::time::*;

pub struct StartRound;

pub const SUMMARY_DURATION: f32 = 15.0;

impl AlarmProcessor for StartRound {
    fn alarm(&self, events: &Vector<Event>) -> Option<Alarm> {
        if !matches!(events.last(), Some(Event::RaceFinished { .. })) {
            return None;
        }

        if projections::game_finished(events) {
            return None;
        }

        Some(Alarm(Duration::from_secs_f32(SUMMARY_DURATION)))
    }
}

impl Processor for StartRound {
    fn process(&self, events: &Vector<Event>) -> Option<Command> {
        let Some(Event::RaceFinished { time, .. }) = events.last() else {
            return None;
        };

        if projections::game_finished(events) {
            return None;
        }

        if SystemTime::now()
            >= UNIX_EPOCH
                + Duration::from_secs(*time as u64)
                + Duration::from_secs_f32(SUMMARY_DURATION - 1.)
        {
            return Some(Command::StartRound(()));
        }

        None
    }
}
