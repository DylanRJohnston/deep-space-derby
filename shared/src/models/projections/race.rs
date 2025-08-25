use std::f32::consts::PI;

use im::Vector;
use rand::{
    Rng, SeedableRng,
    distributions::{Uniform, WeightedIndex},
    prelude::Distribution,
    rngs::StdRng,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::models::{
    events::Event,
    monsters::Monster,
    projections::{game_id, monsters, round},
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RaceResults {
    pub first: Uuid,
    pub second: Uuid,
    pub third: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Jump {
    pub monster_id: usize,
    pub start: f32,
    pub end: f32,
    pub distance: f32,
}

impl PartialOrd for Jump {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Jump {}

impl Ord for Jump {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.start.partial_cmp(&other.start) {
            Some(ord) => ord,
            None => panic!("Failed to compare start times"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RacerParams {
    pub start: f32,
    pub end_time: f32,
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Racer {
    // Linear Paramters
    gradient: f32,
    linear_offset: f32,

    // Sinusoidal Parameters
    amplitude: f32,
    frequency: f32,
    phase: f32,
}

const RACE_TRACK_LENGTH: f32 = 10.0;

impl From<RacerParams> for Racer {
    fn from(params: RacerParams) -> Self {
        let linear_offset = params.start - params.amplitude * (params.phase).sin();
        let gradient = (RACE_TRACK_LENGTH
            - linear_offset
            - params.amplitude * (params.frequency * params.end_time + params.phase).sin())
            / params.end_time;

        Self {
            gradient,
            linear_offset,
            amplitude: params.amplitude,
            frequency: params.frequency,
            phase: params.phase,
        }
    }
}

impl Racer {
    pub fn sample(&self, time: f32) -> f32 {
        self.gradient * time
            + self.linear_offset
            + self.amplitude * (self.frequency * time + self.phase).sin()
    }
}

const STAT_TABLE: [f32; 11] = [-1.0, -0.5, 0.0, 0.3, 0.4, 0.5, 0.6, 0.75, 0.9, 1.0, 1.2];

pub fn results(monsters: &[Monster; 3], seed: u32) -> (RaceResults, Vec<Jump>) {
    let mut rng = StdRng::seed_from_u64(seed as u64);

    let mut weights = WeightedIndex::new(monsters.map(
        |Monster {
             strength,
             dexterity,
             starting_position,
             ..
         }| {
            (STAT_TABLE[strength.clamp(0, 10) as usize]
                + STAT_TABLE[dexterity.clamp(0, 10) as usize]
                + starting_position)
                .clamp(0.001, 3.0)
        },
    ))
    .unwrap();

    let first = weights.sample(&mut rng);
    weights.update_weights(&[(first, &0.0)]).unwrap();

    let second = weights.sample(&mut rng);
    weights.update_weights(&[(second, &0.0)]).unwrap();

    let third = weights.sample(&mut rng);

    let times = {
        let first_time = 8.0 + 2.0 * rng.r#gen::<f32>();
        let second_time = first_time + 0.1 + (rng.r#gen::<f32>() + 1.0) / 2.0;
        let third_time = second_time + 0.1 + (rng.r#gen::<f32>() + 1.0) / 2.0;

        [first_time, second_time, third_time]
    };

    let mut jumps = Vec::new();

    for (place, &index) in [first, second, third].iter().enumerate() {
        let monster = monsters[index];

        let racer = Racer::from(RacerParams {
            start: monster.starting_position,
            end_time: times[place],
            amplitude: 1.0,
            frequency: (2.0 + rng.r#gen::<f32>()) * PI / times[place],
            phase: PI * (rng.r#gen::<f32>() + 1.0),
        });

        let mut time = 0.;

        let dexterity = (monster.dexterity.clamp(0, 10) as f32) / 10.;

        loop {
            let jump_time = {
                let lower = f32::max(1.25 - dexterity, 0.0);
                let upper = f32::max(1.0 - dexterity, 0.00);

                0.1 + (lower + rng.r#gen::<f32>() * (upper - lower))
            };

            let mut distance = racer.sample(time + jump_time).min(10.0);

            if distance == 10.0 {
                distance = 10.2;
            }

            jumps.push(Jump {
                monster_id: index,
                start: time,
                end: time + jump_time,
                distance,
            });

            time += jump_time;

            if distance >= 10.0 {
                break;
            }
        }
    }

    jumps.sort();

    let mut finishes = jumps
        .iter()
        .filter(|item| item.distance >= 10.)
        .collect::<Vec<_>>();

    finishes.sort_by(|a, b| a.end.partial_cmp(&b.end).unwrap());

    (
        RaceResults {
            first: monsters[finishes[0].monster_id].uuid,
            second: monsters[finishes[1].monster_id].uuid,
            third: monsters[finishes[2].monster_id].uuid,
        },
        jumps,
    )
}

pub fn race_duration(events: &Vector<Event>) -> f32 {
    let race_seed = race_seed(events);
    let monsters = monsters(events, race_seed);

    let (_, jumps) = results(&monsters, race_seed);

    jumps.last().unwrap().end
}

// Have to use u32 instead of u64 because JS can't handle u64
#[instrument(skip_all)]
pub fn race_seed_for_round(events: &Vector<Event>, round: u32) -> u32 {
    let game_id = u32::from_be_bytes(game_id(events).bytes().as_chunks::<4>().0[0]);
    let seed = game_id.wrapping_add(round);

    seed
}

pub fn race_seed(events: &Vector<Event>) -> u32 {
    race_seed_for_round(events, round(events))
}

#[cfg(test)]
mod test {
    use quickcheck_macros::quickcheck;

    use crate::models::{
        monsters::MONSTERS,
        projections::race::{self, RACE_TRACK_LENGTH, Racer, RacerParams},
    };

    #[quickcheck]
    pub fn same_outcome_for_same_seed(seed: u32) -> bool {
        let monsters = &[MONSTERS[0], MONSTERS[2], MONSTERS[3]];

        race::results(monsters, seed) == race::results(monsters, seed)
    }

    macro_rules! assert_racer {
        ($racer:expr, $start:expr, $end:expr) => {
            assert_eq!(
                $racer.sample(0.0),
                $start,
                "Racer should start at {}",
                $start
            );
            assert!(
                $racer.sample($end) - RACE_TRACK_LENGTH < 0.001,
                "Racer should end at {}",
                $end
            );
        };
    }

    #[test]
    fn variable_start_position() {
        for start_position in &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0] {
            let racer = Racer::from(RacerParams {
                start: *start_position,
                end_time: 5.0,
                amplitude: 1.0,
                frequency: 1.0,
                phase: 2.0,
            });

            assert_racer!(racer, *start_position, 5.0);
        }
    }

    #[test]
    fn variable_end_time() {
        for end_time in &[1.0, 2.0, 3.0, 4.0, 5.0] {
            let racer = Racer::from(RacerParams {
                start: 0.0,
                end_time: *end_time,
                amplitude: 1.0,
                frequency: 1.0,
                phase: 2.0,
            });

            assert_racer!(racer, 0.0, *end_time);
        }
    }

    #[test]
    fn variable_amplitude() {
        for amplitude in &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0] {
            let racer = Racer::from(RacerParams {
                start: 0.0,
                end_time: 5.0,
                amplitude: *amplitude,
                frequency: 1.0,
                phase: 2.0,
            });

            assert_racer!(racer, 0.0, 5.0);
        }
    }

    #[test]
    fn variable_frequency() {
        for frequency in &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0] {
            let racer = Racer::from(RacerParams {
                start: 0.0,
                end_time: 5.0,
                amplitude: 1.0,
                frequency: *frequency,
                phase: 2.0,
            });

            assert_racer!(racer, 0.0, 5.0);
        }
    }

    #[test]
    fn variable_phase() {
        for phase in &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0] {
            let racer = Racer::from(RacerParams {
                start: 0.0,
                end_time: 5.0,
                amplitude: 1.0,
                frequency: 1.0,
                phase: *phase,
            });

            assert_racer!(racer, 0.0, 5.0);
        }
    }
}
