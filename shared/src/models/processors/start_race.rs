use std::usize;

use im::Vector;
use tracing::instrument;

use super::{Alarm, AlarmProcessor, Processor};
use crate::models::projections;
use crate::models::{commands::Command, events::Event};
use crate::time::*;

pub const PRE_GAME_TIMEOUT: u32 = 90;

pub struct StartRace;

impl AlarmProcessor for StartRace {
    fn alarm(&self, events: &Vector<Event>) -> Option<Alarm> {
        if !matches!(events.last(), Some(Event::RoundStarted { .. })) {
            return None;
        }

        if projections::time_left_in_pregame(events).is_none() {
            return None;
        }

        Some(Alarm(Duration::from_secs(PRE_GAME_TIMEOUT as u64)))
    }
}

impl Processor for StartRace {
    #[instrument(skip_all)]
    fn process(&self, events: &Vector<Event>) -> Option<Command> {
        let last_round_start = events
            .iter()
            .rev()
            .position(|event| matches!(event, Event::RoundStarted { .. }))
            .unwrap_or(usize::MAX);

        let last_race_start = events
            .iter()
            .rev()
            .position(|event| matches!(event, Event::RaceStarted { .. }))
            .unwrap_or(usize::MAX);

        if last_race_start < last_round_start {
            tracing::debug!(?last_race_start, ?last_round_start, "race already started");
            return None;
        }

        if projections::all_players_have_bet(events) {
            tracing::debug!("all players have bet");
            return Some(Command::StartRace(()));
        }

        if projections::time_left_in_pregame(events).is_none() {
            tracing::debug!("no timer for this round");
            return None;
        }

        let Some(Event::RoundStarted { time: start, .. }) = events
            .iter()
            .rev()
            .find(|event| matches!(event, Event::RoundStarted { .. }))
        else {
            tracing::debug!("No round started found");
            return None;
        };

        if SystemTime::now()
            >= UNIX_EPOCH
                + Duration::from_secs(*start as u64)
                + Duration::from_secs(PRE_GAME_TIMEOUT as u64)
        {
            tracing::debug!("Starting race");
            return Some(Command::StartRace(()));
        }

        tracing::debug!("Timer hasn't elapsed yet");
        None
    }
}

#[cfg(test)]
mod test {
    use crate::{test::init_tracing, time::*};

    use im::Vector;
    use uuid::Uuid;

    use crate::models::{
        commands::Command,
        events::{Event, PlacedBet},
        game_id::GameID,
        processors::Processor,
    };

    use super::StartRace;

    #[test]
    fn race_does_not_start() {
        let a: Uuid = Uuid::new_v4();
        let b = Uuid::new_v4();

        let events = Vector::from_iter([
            Event::GameCreated {
                game_id: GameID::random(),
            },
            Event::PlayerJoined {
                session_id: a,
                name: "Test".into(),
            },
            Event::PlayerJoined {
                session_id: b,
                name: "Test".into(),
            },
            Event::PlayerReady { session_id: a },
            Event::PlayerReady { session_id: b },
            Event::start_round_now(),
            Event::PlacedBet(PlacedBet {
                session_id: a,
                monster_id: Uuid::new_v4(),
                amount: 100,
            }),
        ]);

        assert_eq!(None, StartRace.process(&events));
    }

    #[test]
    fn race_does_start() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        let events = Vector::from_iter([
            Event::GameCreated {
                game_id: GameID::random(),
            },
            Event::PlayerJoined {
                session_id: a,
                name: "Test".into(),
            },
            Event::PlayerJoined {
                session_id: b,
                name: "Test".into(),
            },
            Event::PlayerReady { session_id: a },
            Event::PlayerReady { session_id: b },
            Event::start_round_now(),
            Event::PlacedBet(PlacedBet {
                session_id: a,
                monster_id: Uuid::new_v4(),
                amount: 100,
            }),
            Event::PlacedBet(PlacedBet {
                session_id: b,
                monster_id: Uuid::new_v4(),
                amount: 100,
            }),
        ]);

        assert_eq!(Some(Command::StartRace(())), StartRace.process(&events));
    }

    #[test]
    pub fn race_starts_after_timeout() {
        init_tracing();

        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        let events = Vector::from_iter([
            Event::GameCreated {
                game_id: GameID::random(),
            },
            Event::PlayerJoined {
                session_id: a,
                name: "Test".into(),
            },
            Event::PlayerJoined {
                session_id: b,
                name: "Test".into(),
            },
            Event::PlayerReady { session_id: a },
            Event::PlayerReady { session_id: b },
            Event::RoundStarted {
                time: start - 90,
                odds: None,
            },
            Event::RoundStarted {
                time: start - 90,
                odds: None,
            },
            Event::PlacedBet(PlacedBet {
                session_id: a,
                monster_id: Uuid::new_v4(),
                amount: 100,
            }),
        ]);

        assert_eq!(Some(Command::StartRace(())), StartRace.process(&events));
    }
}
