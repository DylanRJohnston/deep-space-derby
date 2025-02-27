use std::{hash::Hash, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    models::{events::OddsExt, monsters::MONSTERS},
    time::*,
};

use rand::{
    distributions::{Uniform, WeightedIndex},
    prelude::Distribution,
    rngs::StdRng,
    Rng, SeedableRng,
};

use super::{
    cards::{Card, Target},
    events::{Event, Odds, Payout, PlacedBet, Settings},
    game_code::GameCode,
    monsters::Monster,
    processors::start_race::PRE_GAME_TIMEOUT,
};
use im::{HashMap, OrdMap, Vector};
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

pub fn players(events: &Vector<Event>) -> OrdMap<Uuid, PlayerInfo> {
    let mut map = OrdMap::new();

    for event in events {
        match event {
            Event::PlayerJoined {
                name, session_id, ..
            } => {
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

pub fn placed_bets(events: &Vector<Event>) -> OrdMap<Uuid, Vector<PlacedBet>> {
    let mut bets = OrdMap::<Uuid, Vector<PlacedBet>>::new();

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

    if players.is_empty() {
        return false;
    }

    for player in players.keys() {
        if !bets.contains_key(player) {
            return false;
        }
    }

    true
}

#[instrument(skip_all)]
pub fn settings(events: &Vector<Event>) -> Settings {
    match &events[0] {
        Event::GameCreated { settings, .. } => *settings,
        event => {
            tracing::error!(?event, "first event wasn't game created");
            return Settings::default();
        }
    }
}

pub const INFLATION_FACTOR: i32 = 110;

pub fn all_account_balances(events: &Vector<Event>) -> OrdMap<Uuid, i32> {
    let mut accounts = OrdMap::<Uuid, i32>::new();
    let mut bets = Vector::new();

    let mut maybe_odds = None;

    for event in events {
        match event {
            Event::PlayerJoined { session_id, .. } => {
                accounts.insert(*session_id, 1000);
            }
            Event::BoughtCard { session_id, .. } => {
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
                bets.push_back(*bet);
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
            } => match settings(events).payout {
                Payout::Odds => {
                    bets.iter()
                        .filter(|bet| bet.monster_id == *first)
                        .for_each(|bet| {
                            let balance = accounts.entry(bet.session_id).or_default();
                            let payout = maybe_odds.payout(bet.monster_id);

                            *balance += (payout * (bet.amount as f32)) as i32;
                        });

                    bets.clear();
                }
                Payout::Pool => {
                    let total_losses = bets
                        .iter()
                        .copied()
                        .filter(|bet| bet.monster_id != *first)
                        .fold(0, |total, bet| total + bet.amount);

                    let total_wins = bets
                        .iter()
                        .copied()
                        .filter(|bet| bet.monster_id == *first)
                        .fold(0, |total, bet| total + bet.amount);

                    bets.iter()
                        .filter(|bet| bet.monster_id == *first)
                        .for_each(|bet| {
                            let balance = accounts.entry(bet.session_id).or_default();

                            let amount = bet.amount
                                + INFLATION_FACTOR * bet.amount * total_losses / total_wins / 100;

                            *balance += amount;
                        });

                    bets.clear();
                }
            },
            Event::PlayedCard {
                session_id: source_uuid,
                card: Card::Theft,
                target: Target::Player(target_uuid),
            } => {
                assert_ne!(
                    target_uuid, source_uuid,
                    "Theft card cannot be played on self"
                );

                let target = accounts.get(target_uuid).copied().unwrap_or_default();
                let source = accounts.get(source_uuid).copied().unwrap_or_default();

                let amount = ((target as f32) * 0.2) as i32;

                accounts.insert(*target_uuid, target - amount);
                accounts.insert(*source_uuid, source + amount);
            }
            Event::PlayedCard {
                card: Card::Crystals,
                target: Target::Player(target),
                ..
            } => {
                *accounts.entry(*target).or_default() += 1000;
            }
            _ => {}
        };
    }

    accounts
}

pub fn account_balance(events: &Vector<Event>, player: Uuid) -> i32 {
    all_account_balances(events)
        .get(&player)
        .copied()
        .unwrap_or_default()
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

// TODO: There's an awkward duplication of logic here between this and `all_account_balances`
// Unfortunately, there's slightly different with winnings ignoring the original bet but all_account_balances
// needing to restore the original bet amount to the wallet
pub fn winnings(events: &Vector<Event>) -> OrdMap<Uuid, i32> {
    let mut winnings = OrdMap::new();
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
            Event::RaceFinished {
                results: RaceResults { first, .. },
                ..
            } => match settings(events).payout {
                Payout::Odds => {
                    for bet in bets.iter() {
                        let balance = winnings.entry(bet.session_id).or_default();

                        let amount = if bet.monster_id == *first {
                            ((odds.payout(bet.monster_id) - 1.0) * (bet.amount as f32)) as i32
                        } else {
                            -1 * bet.amount
                        };

                        *balance += amount
                    }
                }
                Payout::Pool => {
                    let total_losses = bets
                        .iter()
                        .copied()
                        .filter(|bet| bet.monster_id != *first)
                        .fold(0, |total, bet| total + bet.amount);

                    let total_wins = bets
                        .iter()
                        .copied()
                        .filter(|bet| bet.monster_id == *first)
                        .fold(0, |total, bet| total + bet.amount);

                    bets.iter().for_each(|bet| {
                        let balance = winnings.entry(bet.session_id).or_default();

                        let amount = if bet.monster_id == *first {
                            INFLATION_FACTOR * bet.amount * total_losses / total_wins / 100
                        } else {
                            -bet.amount
                        };

                        *balance += amount;
                    });
                }
            },
            _ => {}
        }
    }

    winnings
}

pub fn all_debt(events: &Vector<Event>) -> HashMap<Uuid, u32> {
    let mut debt = HashMap::new();

    for event in events {
        match event {
            Event::BorrowedMoney { session_id, amount } => {
                *debt.entry(*session_id).or_default() += *amount
            }
            Event::PaidBackMoney { session_id, amount } => {
                *debt.entry(*session_id).or_default() -= *amount
            }
            Event::RaceFinished { .. } => debt
                .iter_mut()
                .for_each(|(_, amount)| *amount = ((*amount as f32) * 1.051) as u32),
            _ => {}
        }
    }

    debt
}

pub fn debt(events: &Vector<Event>, player_id: Uuid) -> u32 {
    all_debt(events)
        .get(&player_id)
        .copied()
        .unwrap_or_default()
}

#[instrument(skip_all)]
pub fn game_id(events: &Vector<Event>) -> GameCode {
    match events.get(0) {
        Some(Event::GameCreated { game_id, .. }) => *game_id,
        None => GameCode::random(),
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

const DECK: [(usize, Card); 12] = [
    (10, Card::Poison),
    (10, Card::ExtraRations),
    (10, Card::TasteTester),
    (10, Card::PsyBlast),
    (10, Card::Meditation),
    (10, Card::TinfoilHat),
    (3, Card::Nepotism),
    (8, Card::Theft),
    (8, Card::Extortion),
    (4, Card::Stupify),
    (4, Card::Scrutiny),
    (5, Card::Crystals),
];

fn bought_cards(events: &Vector<Event>) -> u32 {
    let mut count = 0;

    for event in events {
        match event {
            Event::BoughtCard { .. } => count += 1,
            _ => {}
        }
    }

    count
}

pub fn draw_n_cards_from_deck<const N: usize>(events: &Vector<Event>, seed: u64) -> [Card; N] {
    let dist = WeightedIndex::new(DECK.map(|(weight, _)| weight)).unwrap();
    let race_seed = race_seed_for_round(events, bought_cards(events));

    let mut rng = StdRng::seed_from_u64(race_seed as u64 ^ seed);

    core::array::from_fn(|_| DECK[dist.sample(&mut rng)].1)
}

pub fn initial_cards(events: &Vector<Event>, player: Uuid) -> Vec<Card> {
    match settings(events).starting_cards {
        0 => vec![],
        3 => draw_n_cards_from_deck::<3>(events, player.as_u128() as u64).into(),
        // TODO: Remove this horrible hack
        _ => panic!("unsupported number of starting cards"),
    }
}

pub fn cards_in_hand(events: &Vector<Event>, player: Uuid) -> Vec<Card> {
    let mut cards = OrdMap::<Uuid, Vec<Card>>::new();

    let remove_card_from_hand = |cards: &mut Vec<Card>, card| {
        if let Some(index) = cards.iter().position(|it| *it == card) {
            cards.remove(index);
        }
    };

    for event in events {
        match event {
            Event::PlayerJoined {
                session_id,
                initial_cards,
                ..
            } => {
                cards.insert(*session_id, initial_cards.clone());
            }
            Event::BoughtCard { session_id, card } => {
                cards.entry(*session_id).or_default().push(*card)
            }
            Event::PlayedCard {
                session_id: source,
                card: Card::Extortion,
                target: Target::Player(target),
            } if source != target => {
                let target_cards = cards.entry(*target).or_default();
                let mut removed_cards = (0..2)
                    .filter_map(|_| target_cards.pop())
                    .collect::<Vec<_>>();

                let source_cards = cards.entry(*source).or_default();

                remove_card_from_hand(source_cards, Card::Extortion);
                source_cards.append(&mut removed_cards);
            }
            Event::PlayedCard {
                session_id, card, ..
            } => remove_card_from_hand(cards.entry(*session_id).or_default(), *card),
            _ => {}
        }
    }

    cards.remove(&player).unwrap_or_default()
}

pub fn can_play_more_cards(events: &Vector<Event>, player: Uuid) -> bool {
    let max_cards = match player_count(events) {
        0..=3 => 3,
        4..=8 => 2,
        9.. => 1,
    };

    let mut count = 0;

    // Watch out this iterator is backwards so we can short circuit
    for event in events.iter().rev() {
        match event {
            Event::RoundStarted { .. } => return count < max_cards,
            Event::PlayedCard {
                session_id, card, ..
            } if (*session_id == player && !card.is_free()) => count += 1,
            Event::PlayedCard {
                card: Card::Scrutiny,
                target: Target::MultiplePlayers(targets),
                ..
            } if targets.contains(&player) => return false,
            _ => {}
        }
    }

    false
}

pub fn valid_target_for_card(events: &Vector<Event>, player: Uuid, target: Target) -> bool {
    match target {
        Target::Player(target) => player != target && player_exists(events, target),
        Target::MultiplePlayers(targets) => targets
            .iter()
            .all(|target| valid_target_for_card(events, player, Target::Player(*target))),
        Target::Monster(target) => {
            let race_seed = race_seed(events);
            let monsters = monsters(events, race_seed);

            monsters
                .iter()
                .find(|monster| monster.uuid == target)
                .is_some()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct PlayedMonsterCard {
    pub card: Card,
    pub monster_id: Uuid,
}

pub fn unique_played_monster_cards(events: &Vector<Event>) -> Vec<PlayedMonsterCard> {
    let mut cards = Vec::new();

    for event in events.iter() {
        match event {
            Event::PlayedCard {
                card,
                target: Target::Monster(monster_id),
                ..
            } => cards.push(PlayedMonsterCard {
                card: *card,
                monster_id: *monster_id,
            }),
            Event::RoundStarted { .. } => cards.clear(),
            Event::RaceFinished { .. } => cards.clear(),
            _ => {}
        }
    }

    cards.sort();
    cards.dedup();

    cards
}

pub fn poisoned(cards: &Vec<PlayedMonsterCard>, target: Uuid) -> bool {
    let mut poisoned = false;
    let mut countered = false;

    for card in cards {
        if card.monster_id != target {
            continue;
        }

        match card.card {
            Card::Poison => poisoned = true,
            Card::TasteTester => countered = true,
            _ => {}
        }
    }

    poisoned && !countered
}

pub fn nepotism(cards: &Vec<PlayedMonsterCard>, target: Uuid) -> bool {
    for card in cards {
        if card.monster_id != target {
            continue;
        }

        match card.card {
            Card::Nepotism => return true,
            _ => {}
        }
    }

    false
}

pub fn extra_rations(cards: &Vec<PlayedMonsterCard>, target: Uuid) -> bool {
    let mut extra_rations = false;
    let mut countered = false;

    for card in cards {
        if card.monster_id != target {
            continue;
        }

        match card.card {
            Card::ExtraRations => extra_rations = true,
            Card::TasteTester => countered = true,
            _ => {}
        }
    }

    extra_rations && !countered
}

pub fn psyblast(cards: &Vec<PlayedMonsterCard>, target: Uuid) -> bool {
    let mut effect = false;
    let mut countered = false;

    for card in cards {
        if card.monster_id != target {
            continue;
        }

        match card.card {
            Card::PsyBlast => effect = true,
            Card::TinfoilHat => countered = true,
            _ => {}
        }
    }

    effect && !countered
}

pub fn meditation(cards: &Vec<PlayedMonsterCard>, target: Uuid) -> bool {
    let mut effect = false;
    let mut countered = false;

    for card in cards {
        if card.monster_id != target {
            continue;
        }

        match card.card {
            Card::Meditation => effect = true,
            Card::TinfoilHat => countered = true,
            _ => {}
        }
    }

    effect && !countered
}

pub fn monsters(events: &Vector<Event>, race_seed: u32) -> [Monster; 3] {
    use rand::seq::SliceRandom;

    let mut rng = StdRng::seed_from_u64(race_seed as u64);

    let mut monsters: [Monster; 3] = MONSTERS
        .choose_multiple(&mut rng, 3)
        .copied()
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let played_cards = unique_played_monster_cards(&events);

    for monster in &mut monsters {
        if poisoned(&played_cards, monster.uuid) {
            monster.strength -= 3;
        }

        if extra_rations(&played_cards, monster.uuid) {
            monster.strength += 2;
        }

        if psyblast(&played_cards, monster.uuid) {
            monster.dexterity -= 3;
        }

        if meditation(&played_cards, monster.uuid) {
            monster.dexterity += 2;
        }

        if nepotism(&played_cards, monster.uuid) {
            monster.starting_position += 1.5;
        }
    }

    monsters
}

pub fn pre_race_duration(events: &Vector<Event>) -> Duration {
    let played_card = unique_played_monster_cards(&events).len() as u64;

    Duration::from_secs(3 + 4 * played_card)
}

pub fn game_finished(events: &Vector<Event>) -> bool {
    let mut rounds = 0;

    for event in events.iter() {
        match event {
            Event::RaceFinished { .. } => rounds += 1,
            Event::GameFinished => return true,
            _ => {}
        }
    }

    rounds >= 10
}

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

const BASE_JUMP_TIME: f32 = 0.2;
const BASE_JUMP_DISTANCE: f32 = 0.3;

pub fn race(monsters: &[Monster; 3], seed: u32) -> (RaceResults, Vec<Jump>) {
    let mut rng = StdRng::seed_from_u64(seed as u64);

    let mut jumps = Vec::new();

    for (id, monster) in monsters.iter().enumerate() {
        let mut distance = monster.starting_position;
        let mut time = 0.;

        let mut jump_time = 0.;
        let mut jump_distance = 0.;
        let mut counter = 0;

        let dexterity = (monster.dexterity as f32) / 5.;
        let strength = (monster.strength as f32) / 5.;

        loop {
            if counter == 0 {
                counter = Uniform::new(3, 7).sample(&mut rng);
                jump_time = {
                    let lower = f32::max(1.25 - dexterity, 0.0);
                    let upper = f32::max(2.0 - dexterity, 0.00);

                    1.3 * (BASE_JUMP_TIME + 0.5 * (lower + rng.gen::<f32>() * (upper - lower)))
                };
                jump_distance = {
                    let lower = f32::max(f32::powi(strength / 2.0, 2), 0.0);
                    let upper = f32::max(strength, 0.0);

                    0.6 * (BASE_JUMP_DISTANCE + (lower + rng.gen::<f32>() * (upper - lower)))
                };

                // Confusion
                // if rng.gen::<f32>() < 0.25 {
                //     jump_distance *= -1.;
                // }
            }
            counter -= 1;

            distance = f32::min(distance + jump_distance, 10.0);

            if distance == 10.0 {
                distance = 10.2;
            }

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

pub fn race_duration(events: &Vector<Event>) -> f32 {
    let race_seed = race_seed(events);
    let monsters = monsters(events, race_seed);

    let (_, jumps) = race(&monsters, race_seed);

    jumps.last().unwrap().end
}

pub fn time_left_in_pregame(events: &Vector<Event>) -> Option<u64> {
    if events
        .iter()
        .filter(|event| matches!(event, Event::RoundStarted { .. }))
        .count()
        <= 1
    {
        return None;
    }

    let Some(start) = events.iter().rev().find_map(|event| match event {
        Event::RoundStarted { time, .. } => Some(*time),
        _ => None,
    }) else {
        return None;
    };

    match (UNIX_EPOCH
        + Duration::from_secs(start as u64)
        + Duration::from_secs(PRE_GAME_TIMEOUT as u64))
    .duration_since(SystemTime::now())
    {
        Ok(it) => Some(it.as_secs()),
        Err(_) => Some(0),
    }
}

pub fn pre_computed_odds(events: &Vector<Event>) -> Odds {
    let monsters = monsters(events, race_seed(events));

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
pub fn odds(monsters: &[Monster; 3], seed: u32) -> Odds {
    let mut wins = OrdMap::<Uuid, u32>::new();
    let mut rng = StdRng::seed_from_u64(seed as u64);

    for _ in 0..1000 {
        let (results, _) = race(monsters, rng.gen::<u32>());

        *wins.entry(results.first).or_default() += 1;
    }

    Odds(monsters.map(|monster| {
        (
            monster.uuid,
            wins.get(&monster.uuid).copied().unwrap_or_default() as f32 / 1000.,
        )
    }))
}

pub fn maximum_debt(events: &Vector<Event>) -> i32 {
    let mut max_debt = 300;

    for event in events {
        match event {
            Event::RoundStarted { .. } => max_debt += 200,
            _ => {}
        }
    }

    max_debt
}

pub fn results(events: &Vector<Event>) -> Option<RaceResults> {
    for event in events.iter().rev() {
        if let Event::RaceFinished { results, .. } = event {
            return Some(*results);
        }
    }

    None
}

pub fn victim_of_card(events: &Vector<Event>, player: Uuid) -> Option<(Card, String)> {
    let players = players(events);

    match events.last() {
        Some(Event::PlayedCard {
            card,
            session_id,
            target: Target::Player(target),
        }) if *target == player => Some((*card, players.get(session_id).unwrap().name.clone())),
        Some(Event::PlayedCard {
            card,
            session_id,
            target: Target::MultiplePlayers(targets),
        }) if targets.contains(&player) => {
            Some((*card, players.get(session_id).unwrap().name.clone()))
        }
        _ => None,
    }
}

// returns the Some(start time) of the race if its currently in progress, otherwise None
pub fn currently_racing(events: &Vector<Event>) -> Option<u32> {
    match events.iter().rev().find(|event| {
        matches!(
            event,
            Event::RaceStarted { .. } | Event::RaceFinished { .. }
        )
    }) {
        Some(Event::RaceStarted { time }) => Some(*time),
        _ => None,
    }
}

// returns the Some(start time) of the betting round if its currently in progress, otherwise None
pub fn currently_betting(events: &Vector<Event>) -> Option<u32> {
    match events.iter().rev().find(|event| {
        matches!(
            event,
            Event::RoundStarted { .. } | Event::RaceStarted { .. }
        )
    }) {
        Some(Event::RoundStarted { time, .. }) => Some(*time),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use std::default;

    use im::{vector, OrdMap, Vector};
    use uuid::Uuid;

    use crate::models::{
        cards::{Card, Target},
        events::{Event, Payout, PlacedBet, Settings},
        game_code::GameCode,
        projections::{self, RaceResults},
    };

    use super::{all_account_balances, INFLATION_FACTOR};

    use super::{race, MONSTERS};
    use quickcheck_macros::quickcheck;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt().pretty().try_init();
    }

    #[quickcheck]
    pub fn same_outcome_for_same_seed(seed: u32) -> bool {
        let monsters = &[MONSTERS[0], MONSTERS[2], MONSTERS[3]];

        race(monsters, seed) == race(monsters, seed)
    }

    #[test]
    fn empty() {
        let events = Vector::new();

        let accounts = all_account_balances(&events);

        assert_eq!(
            accounts,
            ([] as [(Uuid, i32); 0])
                .into_iter()
                .collect::<OrdMap<Uuid, i32>>()
        )
    }

    #[test]
    fn initial_balances() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                settings: Settings::default(),
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned(),
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned(),
                initial_cards: vec![]
            }
        ];

        let accounts = all_account_balances(&events);

        assert_eq!(
            accounts,
            [(alice, 1000), (bob, 1000)]
                .into_iter()
                .collect::<OrdMap<Uuid, i32>>()
        )
    }

    #[test]
    fn buying_cards() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned(),
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned(),
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Poison
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Poison
            },
            Event::BoughtCard {
                session_id: bob,
                card: Card::Poison
            }
        ];

        let accounts = all_account_balances(&events);

        assert_eq!(
            accounts,
            [(alice, 800), (bob, 900)]
                .into_iter()
                .collect::<OrdMap<Uuid, i32>>()
        )
    }

    #[test]
    fn borrowing_money() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned(),
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned(),
                initial_cards: vec![]
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
                .collect::<OrdMap<Uuid, i32>>()
        )
    }

    #[test]
    fn paying_back_money() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned(),
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned(),
                initial_cards: vec![]
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
                .collect::<OrdMap<Uuid, i32>>()
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
                settings: Settings::default()
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned(),
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned(),
                initial_cards: vec![]
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
                .collect::<OrdMap<Uuid, i32>>()
        )
    }

    #[test]
    fn winning_money_pool() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();
        let monster_c = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                settings: Settings {
                    payout: Payout::Pool,
                    ..Default::default()
                }
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned(),
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned(),
                initial_cards: vec![]
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
            [(alice, 1550), (bob, 500)]
                .into_iter()
                .collect::<OrdMap<Uuid, i32>>()
        )
    }

    #[test]
    fn winning_money_odds() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();
        let monster_c = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                settings: Settings {
                    payout: Payout::Odds,
                    ..Default::default()
                }
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned(),
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned(),
                initial_cards: vec![]
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
            [(alice, 1400), (bob, 500)]
                .into_iter()
                .collect::<OrdMap<Uuid, i32>>()
        )
    }

    #[test]
    fn multiple_rounds_pool() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();
        let monster_c = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                settings: Settings {
                    payout: Payout::Pool,
                    ..Default::default()
                }
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned(),
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned(),
                initial_cards: vec![]
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
            [(alice, 1250), (bob, 830)]
                .into_iter()
                .collect::<OrdMap<Uuid, i32>>()
        )
    }

    #[test]
    fn multiple_rounds_odds() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();
        let monster_c = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                settings: Settings {
                    payout: Payout::Odds,
                    ..Default::default()
                }
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned(),
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned(),
                initial_cards: vec![]
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
                .collect::<OrdMap<Uuid, i32>>()
        )
    }

    #[test]
    fn all_together_pool() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();
        let monster_c = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                settings: Settings {
                    payout: Payout::Pool,
                    ..Default::default()
                }
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned(),
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned(),
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Poison
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
            Event::BoughtCard {
                session_id: bob,
                card: Card::Poison
            },
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
            [(alice, 1250), (bob, 730)]
                .into_iter()
                .collect::<OrdMap<Uuid, i32>>()
        )
    }

    #[test]
    fn all_together_odds() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();
        let monster_c = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: "ABCDEF".try_into().unwrap(),
                settings: Settings {
                    payout: Payout::Odds,
                    ..Default::default()
                }
            },
            Event::PlayerJoined {
                session_id: bob,
                name: "Bob".to_owned(),
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                session_id: alice,
                name: "Alice".to_owned(),
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Poison
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
            Event::BoughtCard {
                session_id: bob,
                card: Card::Poison
            },
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
            [(alice, 1100), (bob, 1800)]
                .into_iter()
                .collect::<OrdMap<Uuid, i32>>()
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
                game_id: GameCode::random(),
                settings: Settings {
                    payout: Payout::Pool,
                    ..Default::default()
                }
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                name: "Bob".into(),
                session_id: bob,
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                name: "Carol".into(),
                session_id: carol,
                initial_cards: vec![]
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
            OrdMap::from([(alice, 10), (bob, 110), (carol, -100)].as_ref())
        );
    }

    #[test]
    fn poison_lowers_strength() {
        init_tracing();

        let alice = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Poison
            },
            Event::RoundStarted {
                time: 0,
                odds: None
            },
        ];

        let monsters = super::monsters(&events, 0);

        let mut post_poison_events = events.clone();
        post_poison_events.push_back(Event::PlayedCard {
            session_id: alice,
            card: Card::Poison,
            target: Target::Monster(monsters[0].uuid),
        });

        let post_poison_monsters = super::monsters(&post_poison_events, 0);

        assert_eq!(monsters[0].strength - 3, post_poison_monsters[0].strength);
        assert_eq!(monsters[1].strength, post_poison_monsters[1].strength);
        assert_eq!(monsters[2].strength, post_poison_monsters[2].strength);
    }

    #[test]
    fn poison_countered_by_taste_tester() {
        init_tracing();

        let alice = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Poison
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::TasteTester
            },
            Event::RoundStarted {
                time: 0,
                odds: None
            },
        ];

        let monsters = super::monsters(&events, 0);

        let mut post_poison_events = events.clone();
        post_poison_events.push_back(Event::PlayedCard {
            session_id: alice,
            card: Card::Poison,
            target: Target::Monster(monsters[0].uuid),
        });
        post_poison_events.push_back(Event::PlayedCard {
            session_id: alice,
            card: Card::TasteTester,
            target: Target::Monster(monsters[0].uuid),
        });

        let post_poison_monsters = super::monsters(&post_poison_events, 0);

        assert_eq!(monsters[0].strength, post_poison_monsters[0].strength);
        assert_eq!(monsters[1].strength, post_poison_monsters[1].strength);
        assert_eq!(monsters[2].strength, post_poison_monsters[2].strength);
    }

    #[test]
    fn extra_rations_increases_strength() {
        init_tracing();

        let alice = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::ExtraRations
            },
            Event::RoundStarted {
                time: 0,
                odds: None
            },
        ];

        let monsters = super::monsters(&events, 0);

        let mut post_rations_events = events.clone();
        post_rations_events.push_back(Event::PlayedCard {
            session_id: alice,
            card: Card::ExtraRations,
            target: Target::Monster(monsters[0].uuid),
        });

        let post_poison_monsters = super::monsters(&post_rations_events, 0);

        assert_eq!(monsters[0].strength + 2, post_poison_monsters[0].strength);
        assert_eq!(monsters[1].strength, post_poison_monsters[1].strength);
        assert_eq!(monsters[2].strength, post_poison_monsters[2].strength);
    }

    #[test]
    fn extra_rations_countered_by_taste_tester() {
        init_tracing();

        let alice = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::ExtraRations
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::TasteTester
            },
            Event::RoundStarted {
                time: 0,
                odds: None
            },
        ];

        let monsters = super::monsters(&events, 0);

        let mut post_rations_events = events.clone();
        post_rations_events.push_back(Event::PlayedCard {
            session_id: alice,
            card: Card::ExtraRations,
            target: Target::Monster(monsters[0].uuid),
        });
        post_rations_events.push_back(Event::PlayedCard {
            session_id: alice,
            card: Card::TasteTester,
            target: Target::Monster(monsters[0].uuid),
        });

        let post_poison_monsters = super::monsters(&post_rations_events, 0);

        assert_eq!(monsters[0].strength, post_poison_monsters[0].strength);
        assert_eq!(monsters[1].strength, post_poison_monsters[1].strength);
        assert_eq!(monsters[2].strength, post_poison_monsters[2].strength);
    }

    #[test]
    fn cards_in_hand() {
        init_tracing();

        let alice = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Poison
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::TasteTester
            },
        ];

        let cards_in_hand = super::cards_in_hand(&events, alice);

        assert_eq!(cards_in_hand, vec![Card::Poison, Card::TasteTester]);
    }

    #[test]
    fn starting_cards() {
        init_tracing();

        // Initial cards are sensitive to player UUID & GameCode
        let alice = Uuid::from_u128(1);

        let game = Event::GameCreated {
            game_id: GameCode::try_from("ABCDEF").unwrap(),
            settings: Settings {
                starting_cards: 3,
                ..Default::default()
            },
        };

        let events = vector![
            game.clone(),
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: projections::initial_cards(&vector![game], alice)
            }
        ];

        let cards_in_hand = super::cards_in_hand(&events, alice);

        assert_eq!(
            cards_in_hand,
            vec![Card::TasteTester, Card::Meditation, Card::TinfoilHat]
        );
    }

    #[test]
    fn playing_a_card_removes_it_from_hand() {
        init_tracing();

        let alice = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Poison
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::TasteTester
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::TasteTester
            },
            Event::PlayedCard {
                session_id: alice,
                card: Card::TasteTester,
                target: Target::Monster(alice),
            }
        ];

        let cards_in_hand = super::cards_in_hand(&events, alice);

        assert_eq!(cards_in_hand, vec![Card::Poison, Card::TasteTester]);
    }

    #[test]
    fn drawing_a_card_is_deterministic() {
        let events = vector![Event::GameCreated {
            game_id: GameCode::random(),
            settings: Settings::default()
        }];

        assert_eq!(
            super::draw_n_cards_from_deck::<1>(&events, 0),
            super::draw_n_cards_from_deck::<1>(&events, 0)
        );
    }

    #[test]
    fn theft() {
        init_tracing();

        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                name: "Bob".into(),
                session_id: bob,
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Theft
            },
            Event::PlayedCard {
                session_id: alice,
                card: Card::Theft,
                target: Target::Player(bob),
            }
        ];

        let balances = super::all_account_balances(&events);

        assert_eq!(balances[&alice], 1100);
        assert_eq!(balances[&bob], 800);
    }

    #[test]
    fn extortion_one_card() {
        init_tracing();

        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::try_from("ABCDEF").unwrap(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                name: "Bob".into(),
                session_id: bob,
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Extortion
            },
            Event::BoughtCard {
                session_id: bob,
                card: Card::Poison,
            },
            Event::PlayedCard {
                session_id: alice,
                card: Card::Extortion,
                target: Target::Player(bob),
            }
        ];

        assert_eq!(super::cards_in_hand(&events, alice), vec![Card::Poison]);
        assert_eq!(super::cards_in_hand(&events, bob), vec![]);
    }

    #[test]
    fn extortion_two_card() {
        init_tracing();

        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::try_from("ABCDEF").unwrap(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                name: "Bob".into(),
                session_id: bob,
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Extortion
            },
            Event::BoughtCard {
                session_id: bob,
                card: Card::Poison,
            },
            Event::BoughtCard {
                session_id: bob,
                card: Card::PsyBlast,
            },
            Event::PlayedCard {
                session_id: alice,
                card: Card::Extortion,
                target: Target::Player(bob),
            }
        ];

        assert_eq!(
            super::cards_in_hand(&events, alice),
            vec![Card::PsyBlast, Card::Poison]
        );
        assert_eq!(super::cards_in_hand(&events, bob), vec![]);
    }

    #[test]
    fn extortion_three_card() {
        init_tracing();

        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::try_from("ABCDEF").unwrap(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                name: "Bob".into(),
                session_id: bob,
                initial_cards: vec![]
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Extortion
            },
            Event::BoughtCard {
                session_id: bob,
                card: Card::Poison,
            },
            Event::BoughtCard {
                session_id: bob,
                card: Card::PsyBlast,
            },
            Event::BoughtCard {
                session_id: bob,
                card: Card::Meditation,
            },
            Event::PlayedCard {
                session_id: alice,
                card: Card::Extortion,
                target: Target::Player(bob),
            }
        ];

        assert_eq!(
            super::cards_in_hand(&events, alice),
            vec![Card::Meditation, Card::PsyBlast]
        );
        assert_eq!(super::cards_in_hand(&events, bob), vec![Card::Poison]);
    }

    #[test]
    fn three_players_numbers_of_cards() {
        init_tracing();

        // The number of cards that can be played per round scales inversely with the number of players.
        // [1-3] -> 3
        // [4-8] -> 2
        // 9+ -> 1

        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();
        let carol = Uuid::new_v4();

        let mut events = vector![
            Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default()
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                name: "Bob".into(),
                session_id: bob,
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                name: "Carol".into(),
                session_id: carol,
                initial_cards: vec![]
            },
            Event::PlayerReady { session_id: alice },
            Event::PlayerReady { session_id: bob },
            Event::PlayerReady { session_id: carol },
            Event::RoundStarted {
                time: 0,
                odds: None
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Poison,
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Poison,
            },
            Event::BoughtCard {
                session_id: alice,
                card: Card::Poison,
            },
            Event::BoughtCard {
                session_id: bob,
                card: Card::Scrutiny,
            },
        ];

        assert!(
            super::can_play_more_cards(&events, alice),
            "Player should be able to play cards initially"
        );

        events.push_back(Event::PlayedCard {
            session_id: alice,
            card: Card::Poison,
            target: Target::Monster(Uuid::new_v4()),
        });
        assert!(
            super::can_play_more_cards(&events, alice),
            "A three player game should allow a player to play at least two card"
        );

        events.push_back(Event::PlayedCard {
            session_id: alice,
            card: Card::Poison,
            target: Target::Monster(Uuid::new_v4()),
        });

        assert!(
            super::can_play_more_cards(&events, alice),
            "A three player game should allow a player to play at least three cards"
        );

        events.push_back(Event::PlayedCard {
            session_id: alice,
            card: Card::Poison,
            target: Target::Monster(Uuid::new_v4()),
        });

        assert!(
            !super::can_play_more_cards(&events, alice),
            "A three player game should allow a player to play no more than three cards"
        );

        events.push_back(Event::PlayedCard {
            session_id: bob,
            card: Card::Scrutiny,
            target: Target::MultiplePlayers(vec![carol]),
        });

        assert!(
            !super::can_play_more_cards(&events, carol),
            "Player should not be able to play cards after being subject to scrutiny"
        );

        events.push_back(Event::RoundStarted {
            time: 1,
            odds: None,
        });
        assert!(
            super::can_play_more_cards(&events, alice),
            "Player should be able to play cards at the start of a new round"
        );
    }

    #[test]
    fn four_to_eight_players_numbers_of_cards() {
        init_tracing();

        let players = (1..=6).map(|_| Uuid::new_v4()).collect::<Vec<_>>();

        let mut events = Vector::from_iter(
            [Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default(),
            }]
            .into_iter()
            .chain(players.iter().clone().flat_map(|id| {
                [
                    Event::PlayerJoined {
                        session_id: *id,
                        name: id.to_string(),
                        initial_cards: vec![],
                    },
                    Event::PlayerReady { session_id: *id },
                ]
            }))
            .chain([Event::RoundStarted {
                time: 0,
                odds: None,
            }]),
        );

        assert!(
            super::can_play_more_cards(&events, players[0]),
            "Player should be able to play cards initially"
        );

        events.push_back(Event::PlayedCard {
            session_id: players[0],
            card: Card::Poison,
            target: Target::Monster(Uuid::new_v4()),
        });
        assert!(
            super::can_play_more_cards(&events, players[0]),
            "A three player game should allow a player to play at least two card"
        );

        events.push_back(Event::PlayedCard {
            session_id: players[0],
            card: Card::Poison,
            target: Target::Monster(Uuid::new_v4()),
        });

        assert!(
            !super::can_play_more_cards(&events, players[0]),
            "A 6 player game should allow a player to play no more than two cards"
        );

        assert!(
            super::can_play_more_cards(&events, players[1]),
            "Other players playing cards should not affect other players ability to play cards"
        );
    }

    #[test]
    fn nine_plus_players_numbers_of_cards() {
        init_tracing();

        let players = (1..=9).map(|_| Uuid::new_v4()).collect::<Vec<_>>();

        let mut events = Vector::from_iter(
            [Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings::default(),
            }]
            .into_iter()
            .chain(players.iter().clone().flat_map(|id| {
                [
                    Event::PlayerJoined {
                        session_id: *id,
                        name: id.to_string(),
                        initial_cards: vec![],
                    },
                    Event::PlayerReady { session_id: *id },
                ]
            }))
            .chain([Event::RoundStarted {
                time: 0,
                odds: None,
            }]),
        );

        assert!(
            super::can_play_more_cards(&events, players[0]),
            "Player should be able to play cards initially"
        );

        events.push_back(Event::PlayedCard {
            session_id: players[0],
            card: Card::Poison,
            target: Target::Monster(Uuid::new_v4()),
        });
        assert!(
            !super::can_play_more_cards(&events, players[0]),
            "A nine player game should allow only a single card to be played"
        );

        assert!(
            super::can_play_more_cards(&events, players[1]),
            "Other players playing cards should not affect other players ability to play cards"
        );
    }

    #[test]
    fn winnings_proportional_to_loss() {
        init_tracing();

        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();
        let carol = Uuid::new_v4();

        let monster_a = Uuid::new_v4();
        let monster_b = Uuid::new_v4();
        let monster_c = Uuid::new_v4();

        let bets: [(&str, &[(Uuid, Uuid, i32)], [(Uuid, i32); 3]); 5] = [
            (
                "winner takes all",
                &[
                    (alice, monster_a, 100),
                    (bob, monster_b, 200),
                    (carol, monster_c, 300),
                ],
                [
                    (alice, INFLATION_FACTOR * 500 / 100),
                    (bob, -200),
                    (carol, -300),
                ],
            ),
            (
                "winnings split proportionally",
                &[
                    (alice, monster_a, 100),
                    (bob, monster_a, 200),
                    (carol, monster_c, 300),
                ],
                [
                    (alice, INFLATION_FACTOR * 100 / 100),
                    (bob, INFLATION_FACTOR * 200 / 100),
                    (carol, -300),
                ],
            ),
            (
                "bet nothing win nothing",
                &[
                    (alice, monster_a, 0),
                    (bob, monster_a, 200),
                    (carol, monster_c, 300),
                ],
                [
                    (alice, 0),
                    (bob, INFLATION_FACTOR * 300 / 100),
                    (carol, -300),
                ],
            ),
            (
                "win some, loose some",
                &[
                    (alice, monster_a, 100),
                    (alice, monster_b, 100),
                    (bob, monster_a, 100),
                    (carol, monster_c, 300),
                ],
                // A weird consequence of this means you win back some of the money you lost
                [
                    (alice, -100 + INFLATION_FACTOR * 200 / 100),
                    (bob, INFLATION_FACTOR * 200 / 100),
                    (carol, -300),
                ],
            ),
            (
                "infinite money glitch",
                &[
                    (alice, monster_a, 1),
                    (alice, monster_b, 1000),
                    (bob, monster_c, 0),
                    (carol, monster_c, 0),
                ],
                // The inflation factor means if you're the only one loosing and winning, you win back more than you lost
                [
                    (alice, -1000 + INFLATION_FACTOR * 1000 / 100),
                    (bob, 0),
                    (carol, 0),
                ],
            ),
        ];

        let events = vector![
            Event::GameCreated {
                game_id: GameCode::random(),
                settings: Settings {
                    payout: Payout::Pool,
                    ..Default::default()
                }
            },
            Event::PlayerJoined {
                name: "Alice".into(),
                session_id: alice,
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                name: "Bob".into(),
                session_id: bob,
                initial_cards: vec![]
            },
            Event::PlayerJoined {
                name: "Carol".into(),
                session_id: carol,
                initial_cards: vec![]
            },
            Event::PlayerReady { session_id: alice },
            Event::PlayerReady { session_id: bob },
            Event::PlayerReady { session_id: carol },
            Event::RoundStarted {
                time: 0,
                odds: None
            }
        ];

        for (name, bets, results) in bets {
            let mut events = events.clone();

            for (session_id, monster_id, amount) in bets.iter().copied() {
                events.push_back(Event::PlacedBet(PlacedBet {
                    session_id,
                    monster_id,
                    amount,
                }));
            }

            events.push_back(Event::RaceStarted { time: 0 });
            events.push_back(Event::RaceFinished {
                time: 0,
                results: RaceResults {
                    first: monster_a,
                    second: monster_b,
                    third: monster_c,
                },
            });

            let accounts = super::all_account_balances(&events);
            let winnings = super::winnings(&events);

            for (session_id, expected_winnings) in results {
                assert_eq!(
                    1000 + expected_winnings,
                    accounts[&session_id],
                    "account balance: {}",
                    name
                );
                assert_eq!(
                    expected_winnings, winnings[&session_id],
                    "winnings: {}",
                    name
                );
            }
        }
    }
}
