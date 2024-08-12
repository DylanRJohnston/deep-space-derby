use anyhow::bail;
use tracing::instrument;
use uuid::Uuid;

use crate::models::{events::Event, monsters::race, projections};

use super::CommandHandler;

pub type Input = ();

pub struct FinishRace;

impl CommandHandler for FinishRace {
    type Input = ();

    #[instrument(skip(events), err)]
    fn handle(
        session_id: uuid::Uuid,
        events: &im::Vector<Event>,
        input: Self::Input,
    ) -> anyhow::Result<Vec<Event>> {
        if session_id != Uuid::nil() {
            bail!("players may not finish the race");
        }

        if !matches!(events.last(), Some(Event::RaceStarted { .. })) {
            bail!("race can only finish if its in progress");
        }

        let race_seed = projections::race_seed(events);
        let monsters = projections::monsters(race_seed);
        let (results, _) = race(&monsters, race_seed);

        Ok(vec![Event::RaceFinished {
            time: Event::now(),
            results,
        }])
    }
}

#[cfg(test)]
mod test {
    use anyhow::{anyhow, bail};
    use im::vector;
    use uuid::Uuid;

    use crate::models::{
        commands::CommandHandler, events::Event, game_id::GameID, monsters::RaceResults,
    };

    use super::FinishRace;

    #[test]
    fn players_may_not_finish_the_race() -> anyhow::Result<()> {
        let player = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameID::random()
            },
            Event::PlayerJoined {
                session_id: player,
                name: "example".into()
            },
            Event::PlayerReady { session_id: player },
            Event::RoundStarted { time: 0 }
        ];

        assert_eq!(
            "players may not finish the race",
            FinishRace::handle(player, &events, ())
                .err()
                .ok_or_else(|| anyhow!("failed to fail"))?
                .root_cause()
                .to_string(),
        );

        Ok(())
    }

    #[test]
    fn race_can_only_finish_if_its_in_progress() -> anyhow::Result<()> {
        let player = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameID::random()
            },
            Event::PlayerJoined {
                session_id: player,
                name: "example".into()
            },
            Event::PlayerReady { session_id: player },
            Event::RoundStarted { time: 0 },
            Event::RaceStarted { time: 0 },
            Event::RaceFinished {
                time: 0,
                results: RaceResults {
                    first: Uuid::new_v4(),
                    second: Uuid::new_v4(),
                    third: Uuid::new_v4(),
                }
            }
        ];

        assert_eq!(
            "race can only finish if its in progress",
            FinishRace::handle(Uuid::nil(), &events, ())
                .err()
                .ok_or_else(|| anyhow!("failed to fail"))?
                .root_cause()
                .to_string(),
        );

        Ok(())
    }

    #[test]
    fn happy_path() -> anyhow::Result<()> {
        let player = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameID::random()
            },
            Event::PlayerJoined {
                session_id: player,
                name: "example".into()
            },
            Event::PlayerReady { session_id: player },
            Event::RoundStarted { time: 0 },
            Event::RaceStarted { time: 0 },
        ];

        if !matches!(
            FinishRace::handle(Uuid::nil(), &events, ())?.last(),
            Some(Event::RaceFinished { .. }),
        ) {
            bail!("didn't finish race");
        }

        Ok(())
    }
}
