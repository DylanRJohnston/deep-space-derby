use im::Vector;
use tracing::instrument;

use super::{Alarm, AlarmProcessor, ProcessManager};
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

impl ProcessManager for StartRace {
    #[instrument(skip_all)]
    fn process(&self, events: &Vector<Event>) -> Option<Command> {
        let Some(start) = projections::currently_betting(events) else {
            tracing::debug!("no betting in progress");
            return None;
        };

        if projections::all_players_have_bet(events) {
            tracing::debug!("all players have bet");
            return Some(Command::StartRace(()));
        }

        if projections::time_left_in_pregame(events).is_none() {
            tracing::debug!("no timer for this round");
            return None;
        }

        if SystemTime::now()
            >= UNIX_EPOCH
                + Duration::from_secs(start as u64)
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
    use crate::{models::events::Settings, test::init_tracing, time::*};

    use im::Vector;
    use uuid::Uuid;

    use crate::models::{
        commands::Command,
        events::{Event, PlacedBet},
        game_code::GameCode,
        process_managers::ProcessManager,
    };

    use super::StartRace;

    #[test]
    fn race_does_not_start() {
        let a: Uuid = Uuid::new_v4();
        let b = Uuid::new_v4();

        let events = Vector::from_iter([
            Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default(),
            },
            Event::PlayerJoined {
                session_id: a,
                name: "Test".into(),
                initial_cards: vec![],
            },
            Event::PlayerJoined {
                session_id: b,
                name: "Test".into(),
                initial_cards: vec![],
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
                game_id: GameCode::random(),
                settings: Settings::default(),
            },
            Event::PlayerJoined {
                session_id: a,
                name: "Test".into(),
                initial_cards: vec![],
            },
            Event::PlayerJoined {
                session_id: b,
                name: "Test".into(),
                initial_cards: vec![],
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
                game_id: GameCode::random(),
                settings: Settings::default(),
            },
            Event::PlayerJoined {
                session_id: a,
                name: "Test".into(),
                initial_cards: vec![],
            },
            Event::PlayerJoined {
                session_id: b,
                name: "Test".into(),
                initial_cards: vec![],
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
