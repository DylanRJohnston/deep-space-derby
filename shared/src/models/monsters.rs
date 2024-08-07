use std::{cmp::min, ops::Deref};

use rand::{
    distributions::{uniform::UniformFloat, Bernoulli, Distribution, Uniform},
    rngs::StdRng,
    Rng, SeedableRng,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Monster {
    pub name: &'static str,
    pub blueprint_name: &'static str,
    pub uuid: Uuid,
    pub speed: f32,
}

pub const MONSTERS: [Monster; 9] = [
    Monster {
        name: "Cactoro",
        uuid: Uuid::from_u128(0xb19768d8fce94b66a2d7ea84799c0101u128),
        blueprint_name: "library/Monster_Cactoro.glb",
        speed: 7.0,
    },
    Monster {
        name: "Purglehorn",
        uuid: Uuid::from_u128(0x99a7c5d8c06744eeb856df9d6b04c4e8u128),
        blueprint_name: "library/Monster_Alien.glb",
        speed: 3.0,
    },
    Monster {
        name: "Mawshroom",
        uuid: Uuid::from_u128(0xf8a2f4560fa44e89b915f0b0de101a1au128),
        blueprint_name: "library/Monster_Mushnub.glb",
        speed: 6.0,
    },
    Monster {
        name: "Mechapanda",
        uuid: Uuid::from_u128(0x0ef5f3373cea4c9ca6655bd3e7bc4c63u128),
        blueprint_name: "library/Monster_Mech.glb",
        speed: 6.0,
    },
    Monster {
        name: "Finflare",
        uuid: Uuid::from_u128(0x6cb10197a7234cf980f7fb957f7eb9f1u128),
        blueprint_name: "library/Monster_Fish.glb",
        speed: 4.0,
    },
    Monster {
        name: "Green Spiky Thing",
        uuid: Uuid::from_u128(0xcbde634a2d3648f383b3c7e45cc864b7u128),
        blueprint_name: "library/Monster_Green_Spiky.glb",
        speed: 6.0,
    },
    Monster {
        name: "Gallus Cranium",
        uuid: Uuid::from_u128(0x73c68289e1334859a0f4e45883076e10u128),
        blueprint_name: "library/Monster_Pink_Slime.glb",
        speed: 6.0,
    },
    Monster {
        name: "Cluckerhead",
        uuid: Uuid::from_u128(0x9f987f8ff320446e8930740aca46954fu128),
        blueprint_name: "library/Monster_Chicken.glb",
        speed: 3.0,
    },
    Monster {
        name: "Fangmaw",
        uuid: Uuid::from_u128(0xb4775b5b2e1f42debe985d3d7890db0du128),
        blueprint_name: "library/Monster_Yeti.glb",
        speed: 4.0,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Round(pub [f32; 3]);

impl Deref for Round {
    type Target = [f32; 3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaceResults {
    pub first: Uuid,
    pub second: Uuid,
    pub third: Uuid,
    pub rounds: Vec<Round>,
}

pub fn race(monsters: &[&Monster; 3], seed: u32) -> RaceResults {
    tracing::info!(?monsters);

    // let dist = Bernoulli::new(1.0 / 6.0).unwrap();
    let mut rng = StdRng::seed_from_u64(seed as u64);

    let mut rounds = Vec::new();

    let mut places = [4; 3];
    let mut speed = [(0, 0.0); 3];
    let mut total = [0.; 3];
    let mut place = 0;

    loop {
        let mut round = Round([0.; 3]);

        for (index, monster) in monsters.iter().enumerate() {
            speed[index].0 -= 1;

            // Sample less causes more variation which is important if we bounce more
            if speed[index].0 <= 0 {
                speed[index].0 = rng.sample(Uniform::new(2, 4));
                speed[index].1 = 0.3 * rng.sample(Uniform::new(0.5, 1.0))
                    + (monster.speed / 6.0) * rng.sample(Uniform::new(0.0, 1.0));
            }

            let moves = speed[index].1;

            round.0[index] = f32::max(f32::min(moves, 10. - total[index]), 0.0);
            total[index] += moves;

            if total[index] >= 10.0 && places.iter().all(|place| *place != index) {
                places[place] = index;
                place += 1;
            }
        }

        rounds.push(round);

        if place == 3 {
            break;
        }
    }

    RaceResults {
        first: monsters[places[0]].uuid,
        second: monsters[places[1]].uuid,
        third: monsters[places[2]].uuid,
        rounds,
    }
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
