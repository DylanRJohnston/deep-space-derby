use std::time::Duration;

use anyhow::Result;
use im::Vector;
use race_timeout::FinishRace;
use start_game::StartGame;
use start_race::StartRace;

use super::{
    commands::{Command, CommandHandler},
    events::Event,
};

pub mod race_timeout;
pub mod start_game;
pub mod start_race;

const PROCESSORS: [&'static dyn Processor; 3] = [&StartGame, &StartRace, &FinishRace];
const ALARMS: [&'static dyn AlarmProcessor; 2] = [&StartRace, &FinishRace];

pub trait Processor: Send + Sync + 'static {
    fn process(&self, events: &Vector<Event>) -> Option<Command>;
}

pub trait AlarmProcessor: Send + Send + 'static {
    fn alarm(&self, events: &Vector<Event>) -> Option<Alarm>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alarm(pub Duration);

pub fn run_processors(events: &Vector<Event>) -> Result<(Vec<Event>, Option<Alarm>)> {
    let mut events = events.clone();
    let mut output = vec![];
    let mut output_alarm = None;

    'outer: loop {
        for processor in PROCESSORS {
            let Some(command) = processor.process(&events) else {
                continue;
            };

            for event in Command::handle(uuid::Uuid::nil(), &events, command)? {
                events.push_back(event.clone());
                output.push(event);
            }

            // A processor triggered a command, start from the beginning again
            continue 'outer;
        }

        for alarm in ALARMS {
            let Some(alarm) = alarm.alarm(&events) else {
                continue;
            };

            if output_alarm.is_some() {
                tracing::warn!("multiple alarms output in a single processor step");
            }

            output_alarm = Some(alarm);
        }

        // No processor triggered a command, stop now
        break;
    }

    Ok((output, output_alarm))
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use anyhow::bail;
    use im::Vector;
    use uuid::Uuid;

    use crate::models::{
        events::Event,
        game_id::GameID,
        processors::{run_processors, Alarm},
    };

    #[test]
    fn processor_trigger() -> anyhow::Result<()> {
        let a = Uuid::new_v4();

        let events = Vector::from_iter([
            Event::GameCreated {
                game_id: GameID::random(),
            },
            Event::PlayerJoined {
                session_id: a,
                name: "Test".into(),
            },
            Event::PlayerReady { session_id: a },
        ]);

        if !matches!(&run_processors(&events)?.0[..], [Event::GameStarted { .. }]) {
            bail!("didn't match");
        };

        Ok(())
    }

    #[test]
    fn pre_game_timeout() -> anyhow::Result<()> {
        let a = Uuid::new_v4();

        let events = Vector::from_iter([
            Event::GameCreated {
                game_id: GameID::random(),
            },
            Event::PlayerJoined {
                session_id: a,
                name: "Test".into(),
            },
            Event::PlayerReady { session_id: a },
            Event::GameStarted { start: 0 },
        ]);

        assert_eq!(
            Some(Alarm(Duration::from_secs_f32(90.))),
            run_processors(&events)?.1
        );

        Ok(())
    }

    #[test]
    fn race_alarm_set() -> anyhow::Result<()> {
        let a = Uuid::new_v4();

        let events = Vector::from_iter([
            Event::GameCreated {
                game_id: GameID::random(),
            },
            Event::PlayerJoined {
                session_id: a,
                name: "Test".into(),
            },
            Event::PlayerReady { session_id: a },
            Event::start_game_now(),
            Event::start_race_now(),
        ]);

        if !matches!(run_processors(&events)?.1, Some(Alarm(_))) {
            bail!("race alarm didn't set");
        }

        Ok(())
    }

    #[test]
    fn race_finishes_automatically() -> anyhow::Result<()> {
        let a = Uuid::new_v4();

        let now = Event::now();

        let events = Vector::from_iter([
            Event::GameCreated {
                game_id: GameID::random(),
            },
            Event::PlayerJoined {
                session_id: a,
                name: "Test".into(),
            },
            Event::PlayerReady { session_id: a },
            Event::GameStarted { start: now },
            Event::RaceStarted { start: now - 60 },
        ]);

        if !matches!(&run_processors(&events)?.0[..], [Event::RaceFinished(_)]) {
            bail!("race didn't finish");
        }

        Ok(())
    }
}
