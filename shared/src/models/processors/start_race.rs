use std::time::{Duration, SystemTime, UNIX_EPOCH};

use im::Vector;

use crate::models::{commands::Command, events::Event, projections};

use super::{Alarm, AlarmProcessor, Processor};

pub const PRE_GAME_TIMEOUT: u32 = 90;

pub struct StartRace;

impl AlarmProcessor for StartRace {
    fn alarm(&self, events: &Vector<Event>) -> Option<Alarm> {
        if !matches!(events.last(), Some(Event::GameStarted { .. })) {
            return None;
        }

        Some(Alarm(Duration::from_secs(PRE_GAME_TIMEOUT as u64)))
    }
}

impl Processor for StartRace {
    fn process(&self, events: &Vector<Event>) -> Option<Command> {
        if !matches!(events.last(), Some(Event::PlacedBet(_))) {
            return None;
        }

        if projections::all_players_have_bet(events) {
            return Some(Command::StartRace(()));
        }

        let Some(Event::GameStarted { start }) = events
            .iter()
            .rev()
            .find(|event| matches!(event, Event::GameStarted { .. }))
        else {
            return None;
        };

        if SystemTime::now()
            >= UNIX_EPOCH
                + Duration::from_secs(*start as u64)
                + Duration::from_secs(PRE_GAME_TIMEOUT as u64)
        {
            return Some(Command::StartRace(()));
        }

        None
    }
}

#[cfg(test)]
mod test {
    use std::time::{SystemTime, UNIX_EPOCH};

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
            Event::start_game_now(),
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
            Event::start_game_now(),
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
            Event::GameStarted { start: start - 90 },
            Event::PlacedBet(PlacedBet {
                session_id: a,
                monster_id: Uuid::new_v4(),
                amount: 100,
            }),
        ]);

        assert_eq!(Some(Command::StartRace(())), StartRace.process(&events));
    }
}