use std::{
    cmp::{max, min},
    str::FromStr,
};

use im::Vector;
use leptos::{
    leptos_dom::{logging::console_log, IntoFragment},
    *,
};
use uuid::Uuid;

use crate::{
    models::{
        commands::{place_bets, PlaceBets},
        monsters::Monster,
        projections,
    },
    server_fns::server_fn,
    utils::{use_events, use_game_id, use_session_id},
};

#[component]
pub fn creature_card(
    name: &'static str,
    amount: RwSignal<i32>,
    available_money: Signal<i32>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let increment_size = 100;

    let set_bet = move |input: i32| {
        // Available money reads all the bets, and so it needs to be called before amount.update for RWLock reasons
        let available_money = available_money();

        amount.update(|amount| {
            *amount = max(*amount + min(input - *amount, available_money), 0);
        });
    };

    let decrement = move |_| set_bet(amount() - 100);
    let increment = move |_| set_bet(amount() + 100);

    let arbitrary_amount = move |ev| {
        set_bet(event_target_value(&ev).parse().unwrap_or_default());
    };

    view! {
        <div class="creature-container">
            <img class="creature-avatar"/>
            <h3>{name}</h3>
            <div class="betting-row">
                {move || {
                    if disabled() {
                        view! {
                            "Bet "
                            {amount}
                        }
                    } else {
                        view! {
                            <button
                                on:click=decrement
                                disabled=move || (disabled() || amount() <= 0)
                            >
                                "-"
                                {increment_size}
                            </button>
                            <input
                                type="number"
                                prop:value=amount
                                on:input=arbitrary_amount
                                disabled=disabled
                            />
                            <button
                                on:click=increment
                                disabled=move || (disabled() || available_money() <= 0)
                            >
                                "+"
                                {increment_size}
                            </button>
                        }
                    }
                }}

            </div>
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Bet {
    name: &'static str,
    monster_id: Uuid,
    amount: RwSignal<i32>,
}

#[component]
pub fn pre_game() -> impl IntoView {
    let game_id = use_game_id();
    let player_id = use_session_id();
    let events = use_events();

    let minimum_bet = move || projections::minimum_bet(events());

    let account_balance = (move || {
        projections::account_balance(&events())
            .get(&player_id)
            .cloned()
            .unwrap_or_default()
    })
    .into_signal();

    let monsters = projections::monsters(&events.get_untracked());

    let bets = monsters
        .iter()
        .map(|Monster { name, uuid, .. }| Bet {
            name,
            monster_id: *uuid,
            amount: create_rw_signal(0),
        })
        .collect::<Vec<_>>();

    let sum_of_bets = Signal::derive({
        let bets = bets.clone();
        move || bets.iter().map(|bet| (bet.amount)()).sum::<i32>()
    });

    let available_money = Signal::derive(move || max(account_balance() - sum_of_bets(), 0));

    let place_bets = {
        let game_id = game_id.clone();

        create_action({
            let bets = bets.clone();
            move |_: &()| {
                server_fn::<PlaceBets>(
                    &game_id,
                    &place_bets::Input {
                        bets: {
                            let bets = bets
                                .iter()
                                .map(|bet| place_bets::Bet {
                                    monster_id: bet.monster_id,
                                    amount: (bet.amount)(),
                                })
                                .collect();
                            console_log(&format!("{:#?}", bets));
                            bets
                        },
                    },
                )
            }
        })
    };

    let placed_bets = create_memo(move |_| {
        projections::placed_bets(&events())
            .get(&player_id)
            .cloned()
            .unwrap_or_default()
    });

    create_effect({
        let bets = bets.clone();
        move |_| {
            let placed_bets = placed_bets();

            if placed_bets.is_empty() {
                return;
            }

            for placed_bet in placed_bets {
                for bet in bets.iter() {
                    if bet.monster_id == placed_bet.monster_id {
                        bet.amount.set(placed_bet.amount);
                    }
                }
            }
        }
    });

    let disabled = Signal::derive(move || !placed_bets().is_empty());

    view! {
        <div class="player-pregame-container">
            <div class="top-row">
                <div>"Lobby: " {game_id}</div>
                <div>"Minimum Bet: " {minimum_bet}</div>
            </div>
            <div class="account-line">
                <div>"Money"</div>
                <div>{account_balance}</div>
                <div>"Debt"</div>
                <div>0</div>
                <div>"Bets"</div>
                <div>{sum_of_bets}</div>
                <div>"Score"</div>
                <div>{account_balance}</div>
                <div>"Rank"</div>
                <div>"#1"</div>
            </div>
            <div class="creature-line">
                <For
                    each=move || bets.clone()
                    key=|it| it.clone()
                    children=move |Bet { name, amount, .. }| {
                        view! { <CreatureCard name amount available_money disabled=disabled/> }
                    }
                />

            </div>
            <div class="action-line">
                <button>"Buy Card"</button>
                <button>"Loan Shark"</button>
                <button on:click=move |_| place_bets.dispatch(()) disabled=disabled>
                    "Place Bets"
                </button>
            </div>
            <div class="card-line">
                <div class="card"></div>
                <div class="card"></div>
                <div class="card"></div>
            </div>
        </div>
    }
}

