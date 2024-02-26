use std::hash::{DefaultHasher, Hash, Hasher};

use crate::models::monsters::MONSTERS;

use super::{
    events::{Event, PlacedBet},
    game_id::GameID,
    monsters::{Monster, Results},
};
use im::{HashMap, Vector};
use rand::SeedableRng;
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

pub fn game_has_started(events: &Vector<Event>) -> bool {
    for event in events {
        if let Event::GameStarted = event {
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

pub fn minimum_bet(events: Vector<Event>) -> i32 {
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

pub fn account_balance(events: &Vector<Event>) -> HashMap<Uuid, i32> {
    let mut accounts = HashMap::<Uuid, i32>::new();
    let mut bets = Vector::new();

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
                    .and_modify(|balance| *balance += amount);
            }
            Event::PaidBackMoney { session_id, amount } => {
                accounts
                    .entry(*session_id)
                    .and_modify(|balance| *balance -= amount);
            }
            Event::PlacedBet(bet) => {
                bets.push_back(bet);
                accounts
                    .entry(bet.session_id)
                    .and_modify(|account| *account -= bet.amount);
            }
            Event::RaceFinished(Results { first, .. }) => {
                for bet in bets.iter() {
                    accounts.entry(bet.session_id).and_modify(|account| {
                        if bet.monster_id == *first {
                            *account += 2 * bet.amount;
                        }
                    });
                }

                bets.clear();
            }
            _ => {}
        };
    }

    accounts
}

pub fn game_id(events: &Vector<Event>) -> GameID {
    match &events[0] {
        Event::GameCreated { game_id, .. } => *game_id,
        _ => "123456".try_into().unwrap(),
    }
}

pub fn round(events: &Vector<Event>) -> u64 {
    let mut round = 0;

    for event in events {
        match event {
            Event::GameStarted => round += 1,
            Event::RaceFinished(_) => round += 1,
            _ => {}
        }
    }

    round
}

pub fn race_seed(events: &Vector<Event>) -> u32 {
    for event in events.iter().rev() {
        if let Event::RaceStarted { seed } = event {
            return *seed;
        }
    }

    0
}

pub fn monsters(events: &Vector<Event>) -> [&'static Monster; 3] {
    use rand::seq::SliceRandom;

    let mut hasher = DefaultHasher::new();
    game_id(events).hash(&mut hasher);

    let seed = hasher.finish() ^ round(events);
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    MONSTERS
        .choose_multiple(&mut rng, 3)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

#[cfg(test)]
mod tests {

    use im::{vector, HashMap, Vector};
    use uuid::Uuid;

    use crate::models::{
        events::{Event, PlacedBet},
        monsters::Results,
    };

    use super::account_balance;

    #[test]
    fn empty() {
        let events = Vector::new();

        let accounts = account_balance(&events);

        assert_eq!(accounts, [].into_iter().collect::<HashMap<Uuid, i32>>())
    }

    #[test]
    fn initial_balances() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                session_id: Uuid::new_v4()
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

        let accounts = account_balance(&events);

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
                session_id: Uuid::new_v4()
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

        let accounts = account_balance(&events);

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
                session_id: Uuid::new_v4()
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

        let accounts = account_balance(&events);

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
                session_id: Uuid::new_v4()
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

        let accounts = account_balance(&events);

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
                session_id: Uuid::new_v4()
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

        let accounts = account_balance(&events);

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
                session_id: Uuid::new_v4()
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
            Event::RaceFinished(Results {
                first: monster_a,
                second: monster_b,
                third: monster_c,
                rounds: Vec::new(),
            })
        ];

        let accounts = account_balance(&events);

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
                session_id: Uuid::new_v4()
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
            Event::RaceFinished(Results {
                first: monster_a,
                second: monster_b,
                third: monster_c,
                rounds: Vec::new(),
            }),
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
            Event::RaceFinished(Results {
                first: monster_b,
                second: monster_a,
                third: monster_c,
                rounds: Vec::new(),
            }),
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
            Event::RaceFinished(Results {
                first: monster_c,
                second: monster_b,
                third: monster_a,
                rounds: Vec::new(),
            })
        ];

        let accounts = account_balance(&events);

        assert_eq!(
            accounts,
            [(alice, 900), (bob, 1300)]
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
                session_id: Uuid::new_v4()
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
            Event::RaceFinished(Results {
                first: monster_a,
                second: monster_b,
                third: monster_c,
                rounds: Vec::new(),
            }),
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
            Event::RaceFinished(Results {
                first: monster_b,
                second: monster_a,
                third: monster_c,
                rounds: Vec::new(),
            }),
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
            Event::RaceFinished(Results {
                first: monster_c,
                second: monster_b,
                third: monster_a,
                rounds: Vec::new(),
            }),
            Event::PaidBackMoney {
                session_id: bob,
                amount: 500
            }
        ];

        let accounts = account_balance(&events);

        assert_eq!(
            accounts,
            [(alice, 900), (bob, 1100)]
                .into_iter()
                .collect::<HashMap<Uuid, i32>>()
        )
    }
}

