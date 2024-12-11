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
        let Some(start) = projections::currently_racing(events) else {
            tracing::debug!("no race in progress");
            return None;
        };

        let duration = projections::pre_race_duration(events)
            + Duration::from_secs_f32(projections::race_duration(events) - 1.);

        tracing::debug!(?duration, now = ?SystemTime::now(), ?start);

        if SystemTime::now() >= UNIX_EPOCH + Duration::from_secs(start as u64) + duration {
            tracing::info!("race finished");
            return Some(Command::FinishRace(()));
        }

        None
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use im::vector;
    use uuid::Uuid;

    use crate::{
        models::{
            commands::Command,
            events::{Event, PlacedBet},
            game_code::GameCode,
            processors::Processor,
        },
        test::init_tracing,
    };

    use super::FinishRace;

    #[test]
    fn race_finish_durability() -> Result<()> {
        init_tracing();

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::random(),
            },
            Event::PlayerJoined {
                session_id: Uuid::new_v4(),
                name: "Test".into(),
            },
            Event::PlayerReady {
                session_id: Uuid::new_v4(),
            },
            Event::RoundStarted {
                time: Event::now(),
                odds: None,
            },
            Event::RaceStarted {
                time: Event::now() - 60,
            },
            Event::PlacedBet(PlacedBet {
                session_id: Uuid::new_v4(),
                monster_id: Uuid::new_v4(),
                amount: 1000
            })
        ];

        assert_eq!(FinishRace.process(&events), Some(Command::FinishRace(())));

        Ok(())
    }
}
