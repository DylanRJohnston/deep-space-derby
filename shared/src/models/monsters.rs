use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    SeedableRng,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Monster {
    pub name: &'static str,
    pub blueprint_name: &'static str,
    pub uuid: Uuid,
    // How fast they jump
    pub speed: f32,
    // How far they jump
    pub strength: f32,
}

impl Monster {
    const DEFAULT: Monster = Monster {
        name: "Unnamed",
        blueprint_name: "no_blueprint_set",
        uuid: Uuid::from_u128(0),
        speed: 1.0,
        strength: 1.0,
    };
}

pub const MONSTERS: [Monster; 9] = [
    Monster {
        name: "Cactoro",
        uuid: Uuid::from_u128(0xb19768d8fce94b66a2d7ea84799c0101u128),
        blueprint_name: "library/Monster_Cactoro.glb",
        ..Monster::DEFAULT
    },
    Monster {
        name: "Purglehorn",
        uuid: Uuid::from_u128(0x99a7c5d8c06744eeb856df9d6b04c4e8u128),
        blueprint_name: "library/Monster_Alien.glb",
        ..Monster::DEFAULT
    },
    Monster {
        name: "Mawshroom",
        uuid: Uuid::from_u128(0xf8a2f4560fa44e89b915f0b0de101a1au128),
        blueprint_name: "library/Monster_Mushnub.glb",
        ..Monster::DEFAULT
    },
    Monster {
        name: "Mechapanda",
        uuid: Uuid::from_u128(0x0ef5f3373cea4c9ca6655bd3e7bc4c63u128),
        blueprint_name: "library/Monster_Mech.glb",
        ..Monster::DEFAULT
    },
    Monster {
        name: "Finflare",
        uuid: Uuid::from_u128(0x6cb10197a7234cf980f7fb957f7eb9f1u128),
        blueprint_name: "library/Monster_Fish.glb",
        ..Monster::DEFAULT
    },
    Monster {
        name: "Green Spiky Thing",
        uuid: Uuid::from_u128(0xcbde634a2d3648f383b3c7e45cc864b7u128),
        blueprint_name: "library/Monster_Green_Spiky.glb",
        ..Monster::DEFAULT
    },
    Monster {
        name: "Gallus Cranium",
        uuid: Uuid::from_u128(0x73c68289e1334859a0f4e45883076e10u128),
        blueprint_name: "library/Monster_Pink_Slime.glb",
        ..Monster::DEFAULT
    },
    Monster {
        name: "Cluckerhead",
        uuid: Uuid::from_u128(0x9f987f8ff320446e8930740aca46954fu128),
        blueprint_name: "library/Monster_Chicken.glb",
        ..Monster::DEFAULT
    },
    Monster {
        name: "Fangmaw",
        uuid: Uuid::from_u128(0xb4775b5b2e1f42debe985d3d7890db0du128),
        blueprint_name: "library/Monster_Yeti.glb",
        ..Monster::DEFAULT
    },
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

const BASE_JUMP_TIME: f32 = 0.4;
const BASE_JUMP_DISTANCE: f32 = 0.1;

pub fn race(monsters: &[&Monster; 3], seed: u32) -> (RaceResults, Vec<Jump>) {
    let mut rng = StdRng::seed_from_u64(seed as u64);

    let mut jumps = Vec::new();

    for (id, monster) in monsters.iter().enumerate() {
        let mut distance = 0.;
        let mut time = 0.;

        let mut jump_time = 0.;
        let mut jump_distance = 0.;
        let mut counter = 0;

        loop {
            if counter == 0 {
                counter = Uniform::new(1, 5).sample(&mut rng);
                jump_time = BASE_JUMP_TIME
                    + Uniform::new(0.0, f32::max(1.3 - monster.speed, 0.01)).sample(&mut rng);
                jump_distance = BASE_JUMP_DISTANCE
                    + 0.75 * Uniform::new(0.0, f32::max(monster.strength, 0.01)).sample(&mut rng);
            }
            counter -= 1;

            distance = f32::min(distance + jump_distance, 10.0);

            jumps.push(Jump {
                monster_id: id,
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

#[cfg(test)]
mod test {

    use super::{race, MONSTERS};
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    pub fn same_outcome_for_same_seed(seed: u32) -> bool {
        let monsters = &[&MONSTERS[0], &MONSTERS[2], &MONSTERS[3]];

        race(monsters, seed) == race(monsters, seed)
    }
}
