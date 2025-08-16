use im::Vector;

use crate::models::{commands::Command, events::Event, projections};

use super::ProcessManager;

pub struct StartGame;

impl ProcessManager for StartGame {
    fn process(&self, events: &Vector<Event>) -> Option<Command> {
        if !matches!(events.last(), Some(Event::PlayerReady { .. })) {
            return None;
        }

        projections::all_players_ready(events).then_some(Command::StartRound(()))
    }
}

#[cfg(test)]
mod test {
    use im::Vector;
    use uuid::Uuid;

    use crate::models::{
        commands::Command,
        events::{Event, Settings},
        game_code::GameCode,
        process_managers::ProcessManager,
    };

    use super::StartGame;

    #[test]
    fn game_does_not_start() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        let mut events = Vector::from_iter([
            Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default(),
            },
            Event::PlayerJoined {
                session_id: a,
                name: "Test".into(),
                initial_cards: vec![],
            },
        ]);

        assert_eq!(None, StartGame.process(&events));

        events.push_back(Event::PlayerJoined {
            session_id: b,
            name: "Test".into(),
            initial_cards: vec![],
        });

        events.push_back(Event::PlayerReady { session_id: a })
    }

    #[test]
    fn game_does_start() {
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
        ]);

        assert_eq!(Some(Command::StartRound(())), StartGame.process(&events));
    }
}
