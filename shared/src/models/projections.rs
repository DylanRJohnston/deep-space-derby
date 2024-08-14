use std::{hash::Hash, time::Duration};

#[cfg(feature = "wasm")]
use web_time::{SystemTime, UNIX_EPOCH};

#[cfg(not(feature = "wasm"))]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::monsters::MONSTERS;

use rand::Rng;

use super::{
    events::{Event, Odds, OddsExt, PlacedBet},
    game_id::GameID,
    monsters::{self, Monster, RaceResults},
    processors::start_race::PRE_GAME_TIMEOUT,
};
use im::{HashMap, Vector};
use rand::{rngs::StdRng, SeedableRng};
use tracing::instrument;
use uuid::Uuid;

pub fn player_count(events: &Vector<Event>) -> usize {
    let mut count = 0;

    for event in events {
        if let Event::PlayerJoined { .. } = event {
            count += 1
        }
    }

    count
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct PlayerInfo {
    pub session_id: Uuid,
    pub name: String,
    pub ready: bool,
}

pub fn players(events: &Vector<Event>) -> HashMap<Uuid, PlayerInfo> {
    let mut map = HashMap::new();

    for event in events {
        match event {
            Event::PlayerJoined { name, session_id } => {
                map.insert(
                    *session_id,
                    PlayerInfo {
                        session_id: *session_id,
                        name: name.clone(),
                        ready: false,
                    },
                );
            }
            Event::ChangedProfile { session_id, name } => {
                if let Some(info) = map.get_mut(session_id) {
                    info.name = name.clone();
                }
            }
            Event::PlayerReady { session_id } => {
                if let Some(info) = map.get_mut(session_id) {
                    info.ready = true
                }
            }
            _ => {}
        }
    }

    map
}

pub fn player_info(events: &Vector<Event>, player: Uuid) -> Option<PlayerInfo> {
    players(events).get(&player).cloned()
}

pub fn game_has_started(events: &Vector<Event>) -> bool {
    for event in events {
        if let Event::RoundStarted { .. } = event {
            return true;
        }
    }

    false
}

pub fn player_exists(events: &Vector<Event>, session_id: Uuid) -> bool {
    for event in events {
        if let Event::PlayerJoined {
            session_id: inner_session_id,
            ..
        } = event
        {
            if session_id == *inner_session_id {
                return true;
            }
        }
    }

    false
}

pub fn all_players_ready(events: &Vector<Event>) -> bool {
    players(events).values().all(|player| player.ready)
}

pub fn minimum_bet(events: &Vector<Event>) -> i32 {
    let mut starting_bet = 100;

    for event in events {
        if let Event::RaceFinished { .. } = event {
            starting_bet += 100
        }
    }

    starting_bet
}

pub fn placed_bets(events: &Vector<Event>) -> HashMap<Uuid, Vector<PlacedBet>> {
    let mut bets = HashMap::<Uuid, Vector<PlacedBet>>::new();

    for event in events {
        match event {
            Event::PlacedBet(bet) => bets.entry(bet.session_id).or_default().push_back(*bet),
            Event::RaceFinished { .. } => bets.clear(),
            _ => {}
        }
    }

    bets
}

pub fn player_has_bet(events: &Vector<Event>, player_id: Uuid) -> bool {
    let mut player_has_bet = false;

    for event in events {
        match event {
            Event::PlacedBet(PlacedBet { session_id, .. }) if player_id == *session_id => {
                player_has_bet = true
            }
            Event::RaceFinished { .. } => player_has_bet = false,
            _ => {}
        }
    }

    player_has_bet
}

pub fn all_players_have_bet(events: &Vector<Event>) -> bool {
    let players = players(events);
    let bets = placed_bets(events);

    for player in players.keys() {
        if !bets.contains_key(player) {
            return false;
        }
    }

    true
}

pub fn all_account_balances(events: &Vector<Event>) -> HashMap<Uuid, i32> {
    let mut accounts = HashMap::<Uuid, i32>::new();
    let mut bets = Vector::new();

    let mut maybe_odds = None;

    for event in events {
        match event {
            Event::PlayerJoined { session_id, .. } => {
                accounts.insert(*session_id, 1000);
            }
            Event::BoughtCard { session_id } => {
                accounts
                    .entry(*session_id)
                    .and_modify(|balance| *balance -= 100);
            }
            Event::BorrowedMoney { session_id, amount } => {
                accounts
                    .entry(*session_id)
                    .and_modify(|balance| *balance += *amount as i32);
            }
            Event::PaidBackMoney { session_id, amount } => {
                accounts
                    .entry(*session_id)
                    .and_modify(|balance| *balance -= *amount as i32);
            }
            Event::PlacedBet(bet) => {
                bets.push_back(bet);
                accounts
                    .entry(bet.session_id)
                    .and_modify(|account| *account -= bet.amount);
            }
            Event::RoundStarted { odds, .. } => {
                maybe_odds = *odds;
            }
            Event::RaceFinished {
                results: RaceResults { first, .. },
                ..
            } => {
                bets.iter()
                    .filter(|bet| bet.monster_id == *first)
                    .for_each(|bet| {
                        let balance = accounts.entry(bet.session_id).or_default();

                        let payout = maybe_odds.payout(bet.monster_id);

                        *balance += (payout * (bet.amount as f32)) as i32;

                        tracing::info!(?bet.amount, ?payout, ?balance);
                    });

                bets.clear();
            }
            _ => {}
        };
    }

    accounts
}

pub fn last_round(events: &Vector<Event>) -> Option<Vector<Event>> {
    let start = events
        .iter()
        .rev()
        .enumerate()
        .find(|(_, event)| matches!(event, Event::RoundStarted { .. }))?
        .0;

    let end = events
        .iter()
        .rev()
        .enumerate()
        .find(|(_, event)| matches!(event, Event::RaceFinished { .. }))
        .map(|(index, _)| index)
        .unwrap_or_else(|| events.len());

    Some(events.clone().slice(start..=end))
}

pub fn winnings(events: &Vector<Event>) -> HashMap<Uuid, i32> {
    let mut winnings = HashMap::new();
    let mut bets = Vec::new();
    let odds = pre_computed_odds(events);

    for event in events {
        match event {
            Event::RoundStarted { .. } => {
                winnings.clear();
                bets.clear();
            }
            Event::PlacedBet(bet) => {
                bets.push(*bet);
            }
            Event::RaceFinished { results, .. } => {
                for bet in bets.iter() {
                    *winnings.entry(bet.session_id).or_default() +=
                        if bet.monster_id == results.first {
                            ((odds.payout(bet.monster_id) - 1.0) * (bet.amount as f32)) as i32
                        } else {
                            -1 * bet.amount
                        }
                }
            }
            _ => {}
        }
    }

    winnings
}

pub fn debt(events: &Vector<Event>, player_id: Uuid) -> u32 {
    let mut debt = 0;

    for event in events {
        match event {
            Event::BorrowedMoney { session_id, amount } if *session_id == player_id => {
                debt += amount
            }
            Event::PaidBackMoney { session_id, amount } if *session_id == player_id => {
                debt -= amount
            }
            Event::RaceFinished { .. } => debt = ((debt as f32) * 1.051) as u32,
            _ => {}
        };
    }

    debt
}

#[instrument(skip_all)]
pub fn game_id(events: &Vector<Event>) -> GameID {
    match events.get(0) {
        Some(Event::GameCreated { game_id, .. }) => *game_id,
        None => GameID::random(),
        Some(event) => {
            tracing::error!(?event, "first event wasn't game_created");
            unreachable!()
        }
    }
}

pub fn round(events: &Vector<Event>) -> u32 {
    let mut round = 0;

    for event in events {
        match event {
            Event::RoundStarted { .. } => round += 1,
            _ => {}
        }
    }

    round
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

pub fn monsters(race_seed: u32) -> [&'static Monster; 3] {
    use rand::seq::SliceRandom;

    let mut rng = rand::rngs::StdRng::seed_from_u64(race_seed as u64);

    MONSTERS
        .choose_multiple(&mut rng, 3)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

pub fn race_duration(events: &Vector<Event>) -> f32 {
    let race_seed = race_seed(events);
    let monsters = monsters(race_seed);

    let (_, jumps) = crate::models::monsters::race(&monsters, race_seed);

    jumps.last().unwrap().end
}

pub fn time_left_in_pregame(events: &Vector<Event>) -> u64 {
    let Some(start) = events
        .iter()
        .rev()
        .find_map(|event| match event {
            Event::RoundStarted { time: start, .. } => Some(start),
            _ => None,
        })
        .copied()
    else {
        return 0;
    };

    match (UNIX_EPOCH
        + Duration::from_secs(start as u64)
        + Duration::from_secs(PRE_GAME_TIMEOUT as u64))
    .duration_since(SystemTime::now())
    {
        Ok(it) => it.as_secs(),
        Err(_) => 0,
    }
}

pub fn pre_computed_odds(events: &Vector<Event>) -> Odds {
    let monsters = monsters(race_seed(events));

    events
        .iter()
        .rev()
        .find_map(|event| match event {
            Event::RoundStarted {
                odds: Some(odds), ..
            } => Some(odds),
            _ => None,
        })
        .copied()
        .unwrap_or_else(|| {
            Odds([
                (monsters[0].uuid, 1. / 3.),
                (monsters[1].uuid, 1. / 3.),
                (monsters[2].uuid, 1. / 3.),
            ])
        })
}
pub fn odds(monsters: &[&Monster; 3], seed: u32) -> Odds {
    let mut wins = HashMap::<Uuid, u32>::new();
    let mut rng = StdRng::seed_from_u64(seed as u64);

    for _ in 0..1000 {
        let (results, _) = monsters::race(monsters, rng.gen::<u32>());

        *wins.entry(results.first).or_default() += 1;
    }

    Odds(monsters.map(|monster| {
        (
            monster.uuid,
            wins.get(&monster.uuid).copied().unwrap_or_default() as f32 / 1000.,
        )
    }))
}

#[cfg(test)]
mod tests {

    use im::{vector, HashMap, Vector};
    use uuid::Uuid;

    use crate::models::{
        events::{Event, PlacedBet},
        game_id::GameID,
        monsters::RaceResults,
    };

    use super::all_account_balances;

    #[test]
    fn empty() {
        let events = Vector::new();

        let accounts = all_account_balances(&events);

        assert_eq!(accounts, [].into_iter().collect::<HashMap<Uuid, i32>>())
    }

    #[test]
    fn initial_balances() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                // session_id: Uuid::new_v4()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned()
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned()
            }
        ];

        let accounts = all_account_balances(&events);

        assert_eq!(
            accounts,
            [(alice, 1000), (bob, 1000)]
                .into_iter()
                .collect::<HashMap<Uuid, i32>>()
        )
    }

    #[test]
    fn buying_cards() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                // session_id: Uuid::new_v4()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned()
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned()
            },
            Event::BoughtCard { session_id: alice },
            Event::BoughtCard { session_id: alice },
            Event::BoughtCard { session_id: bob }
        ];

        let accounts = all_account_balances(&events);

        assert_eq!(
            accounts,
            [(alice, 800), (bob, 900)]
                .into_iter()
                .collect::<HashMap<Uuid, i32>>()
        )
    }

    #[test]
    fn borrowing_money() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                // session_id: Uuid::new_v4()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned()
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned()
            },
            Event::BorrowedMoney {
                session_id: alice,
                amount: 1000
            },
            Event::BorrowedMoney {
                session_id: bob,
                amount: 200
            },
        ];

        let accounts = all_account_balances(&events);

        assert_eq!(
            accounts,
            [(alice, 2000), (bob, 1200)]
                .into_iter()
                .collect::<HashMap<Uuid, i32>>()
        )
    }

    #[test]
    fn paying_back_money() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                // session_id: Uuid::new_v4()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned()
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned()
            },
            Event::BorrowedMoney {
                session_id: alice,
                amount: 1000
            },
            Event::BorrowedMoney {
                session_id: bob,
                amount: 200
            },
            Event::PaidBackMoney {
                session_id: bob,
                amount: 100
            }
        ];

        let accounts = all_account_balances(&events);

        assert_eq!(
            accounts,
            [(alice, 2000), (bob, 1100)]
                .into_iter()
                .collect::<HashMap<Uuid, i32>>()
        )
    }

    #[test]
    fn placing_bets() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                // session_id: Uuid::new_v4()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned()
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned()
            },
            Event::PlacedBet(PlacedBet {
                session_id: alice,
                monster_id: monster_a,
                amount: 200
            }),
            Event::PlacedBet(PlacedBet {
                session_id: bob,
                monster_id: monster_b,
                amount: 500
            })
        ];

        let accounts = all_account_balances(&events);

        assert_eq!(
            accounts,
            [(alice, 800), (bob, 500)]
                .into_iter()
                .collect::<HashMap<Uuid, i32>>()
        )
    }

    #[test]
    fn winning_money() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();
        let monster_c = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                // session_id: Uuid::new_v4()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned()
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned()
            },
            Event::PlacedBet(PlacedBet {
                session_id: alice,
                monster_id: monster_a,
                amount: 200
            }),
            Event::PlacedBet(PlacedBet {
                session_id: bob,
                monster_id: monster_b,
                amount: 500
            }),
            Event::RaceFinished {
                time: Event::now(),
                results: RaceResults {
                    first: monster_a,
                    second: monster_b,
                    third: monster_c,
                }
            }
        ];

        let accounts = all_account_balances(&events);

        assert_eq!(
            accounts,
            [(alice, 1200), (bob, 500)]
                .into_iter()
                .collect::<HashMap<Uuid, i32>>()
        )
    }

    #[test]
    fn multiple_rounds() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();
        let monster_c = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                // session_id: Uuid::new_v4()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned()
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned()
            },
            Event::PlacedBet(PlacedBet {
                session_id: alice,
                monster_id: monster_a,
                amount: 200
            }),
            Event::PlacedBet(PlacedBet {
                session_id: bob,
                monster_id: monster_b,
                amount: 500
            }),
            Event::RaceFinished {
                time: Event::now(),
                results: RaceResults {
                    first: monster_a,
                    second: monster_b,
                    third: monster_c,
                }
            },
            Event::PlacedBet(PlacedBet {
                session_id: alice,
                monster_id: monster_a,
                amount: 250
            }),
            Event::PlacedBet(PlacedBet {
                session_id: bob,
                monster_id: monster_b,
                amount: 500
            }),
            Event::RaceFinished {
                time: Event::now(),
                results: RaceResults {
                    first: monster_b,
                    second: monster_a,
                    third: monster_c,
                }
            },
            Event::PlacedBet(PlacedBet {
                session_id: alice,
                monster_id: monster_b,
                amount: 50
            }),
            Event::PlacedBet(PlacedBet {
                session_id: bob,
                monster_id: monster_c,
                amount: 300
            }),
            Event::RaceFinished {
                time: Event::now(),
                results: RaceResults {
                    first: monster_c,
                    second: monster_b,
                    third: monster_a,
                }
            }
        ];

        let accounts = all_account_balances(&events);

        assert_eq!(
            accounts,
            [(alice, 1100), (bob, 2100)]
                .into_iter()
                .collect::<HashMap<Uuid, i32>>()
        )
    }

    #[test]
    fn all_together() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();
        let monster_c = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                // session_id: Uuid::new_v4()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned()
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned()
            },
            Event::BoughtCard { session_id: alice },
            Event::PlacedBet(PlacedBet {
                session_id: alice,
                monster_id: monster_a,
                amount: 200
            }),
            Event::PlacedBet(PlacedBet {
                session_id: bob,
                monster_id: monster_b,
                amount: 500
            }),
            Event::RaceFinished {
                time: Event::now(),
                results: RaceResults {
                    first: monster_a,
                    second: monster_b,
                    third: monster_c,
                }
            },
            Event::BoughtCard { session_id: bob },
            Event::BorrowedMoney {
                session_id: bob,
                amount: 500
            },
            Event::PlacedBet(PlacedBet {
                session_id: alice,
                monster_id: monster_a,
                amount: 250
            }),
            Event::PlacedBet(PlacedBet {
                session_id: bob,
                monster_id: monster_b,
                amount: 400
            }),
            Event::BorrowedMoney {
                session_id: alice,
                amount: 100
            },
            Event::RaceFinished {
                time: Event::now(),
                results: RaceResults {
                    first: monster_b,
                    second: monster_a,
                    third: monster_c,
                }
            },
            Event::PlacedBet(PlacedBet {
                session_id: alice,
                monster_id: monster_b,
                amount: 50
            }),
            Event::PlacedBet(PlacedBet {
                session_id: bob,
                monster_id: monster_c,
                amount: 300
            }),
            Event::RaceFinished {
                time: Event::now(),
                results: RaceResults {
                    first: monster_c,
                    second: monster_b,
                    third: monster_a,
                }
            },
            Event::PaidBackMoney {
                session_id: bob,
                amount: 500
            }
        ];

        let accounts = all_account_balances(&events);

        assert_eq!(
            accounts,
            [(alice, 900), (bob, 1100)]
                .into_iter()
                .collect::<HashMap<Uuid, i32>>()
        )
    }

    #[test]
    fn winnings() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();
        let carol = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();
        let monster_c = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameID::random()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice
            },
            Event::PlayerJoined {
                name: "Bob".into(),
                session_id: bob
            },
            Event::PlayerJoined {
                name: "Carol".into(),
                session_id: carol
            },
            Event::PlacedBet(PlacedBet {
                session_id: alice,
                monster_id: monster_a,
                amount: 100
            }),
            Event::PlacedBet(PlacedBet {
                session_id: alice,
                monster_id: monster_b,
                amount: 100
            }),
            Event::PlacedBet(PlacedBet {
                session_id: bob,
                monster_id: monster_b,
                amount: 100
            }),
            Event::PlacedBet(PlacedBet {
                session_id: carol,
                monster_id: monster_a,
                amount: 100
            }),
            Event::RaceFinished {
                time: Event::now(),
                results: RaceResults {
                    first: monster_b,
                    second: monster_a,
                    third: monster_c
                }
            }
        ];

        let winnings = super::winnings(&events);

        assert_eq!(
            winnings,
            HashMap::from([(alice, 0_i32), (bob, 100), (carol, -100)].as_ref())
        );
    }
}
