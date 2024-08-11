use std::cmp::{max, min};

use ev::MouseEvent;
use leptos::*;
use uuid::Uuid;

use crate::{
    server_fns::server_fn,
    utils::{use_events, use_game_id, use_session_id},
};
use shared::models::{
    commands::{borrow_money, place_bets, BorrowMoney, PlaceBets},
    monsters::Monster,
    projections,
};

#[component]
pub fn creature_card(
    name: &'static str,
    amount: RwSignal<i32>,
    available_money: Signal<i32>,
) -> impl IntoView {
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
            <h3>{name}</h3>
            <div class="betting-row">
                <button
                    on:click=decrement
                    disabled=move || (amount() <= 0)
                >
                    "-"
                    // {increment_size}
                </button>
                <input
                    type="number"
                    prop:value=amount
                    on:input=arbitrary_amount
                />
                <button
                    on:click=increment
                    disabled=move || (available_money() <= 0)
                >
                    "+"
                    // {increment_size}
                </button>
            </div>
        </div>

    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    let player_info = move || projections::player_info(&events(), player_id);
    let player_name = move || player_info().map(|player| player.name);

    let untracked_events = events.get_untracked();

    // let minimum_bet = projections::minimum_bet(&untracked_events);
    let monsters = projections::monsters(projections::race_seed(&untracked_events));

    let account_balance = (move || {
        projections::all_account_balances(&events())
            .get(&player_id)
            .copied()
            .unwrap_or_default()
    })
    .into_signal();

    let bets = {
        monsters.map(|Monster { name, uuid, .. }| Bet {
            name,
            monster_id: *uuid,
            amount: create_rw_signal(0),
        })
    };

    let debt = move || projections::debt(&events(), player_id);

    let sum_of_bets = Signal::derive(move || bets.iter().map(|bet| (bet.amount)()).sum::<i32>());

    let available_money = Signal::derive(move || max(account_balance() - sum_of_bets(), 0));

    let place_bets = create_action({
        move |_: &()| {
            server_fn::<PlaceBets>(
                game_id,
                &place_bets::Input {
                    bets: {
                        let bets = bets
                            .iter()
                            .filter(|bet| bet.amount.get() > 0)
                            .map(|bet| place_bets::Bet {
                                monster_id: bet.monster_id,
                                amount: (bet.amount)(),
                            })
                            .collect();
                        bets
                    },
                },
            )
        }
    });

    let placed_bets = create_memo(move |_| {
        projections::placed_bets(&events())
            .get(&player_id)
            .cloned()
            .unwrap_or_default()
    });

    create_effect({
        move |_| {
            let placed_bets = placed_bets();

            if placed_bets.is_empty() {
                return;
            }

            for placed_bet in placed_bets {
                for bet in bets {
                    if bet.monster_id == placed_bet.monster_id {
                        bet.amount.set(placed_bet.amount);
                    }
                }
            }
        }
    });

    let (bets_modal, toggle_bets_modal) = {
        let (read, write) = create_signal(false);

        (read, move || write(!read()))
    };

    let (loan_modal, toggle_loan_modal) = {
        let (read, write) = create_signal(false);

        (read, move || write(!read()))
    };

    view! {
        <div class="pre-game-container">
            <div class="profile-image">"Profile Image"</div>
            <div class="player-info">
                <h2>{player_name}</h2>
                <p>"Funds = $"{available_money}</p>
                <p>"Debt = $"{debt}</p>
            </div>
            <div class="action-grid">
                <div class="placeholder-image">
                    <p>"Image"</p>
                    <p class="emoji">"🃏"</p>
                </div>
                <div class="placeholder-image">
                    <p>"Image"</p>
                    <p class="emoji">"🦈"</p>
                </div>
                <div class="action">
                    <p>"Buy a card"</p>
                    <p>"$(100)"</p>
                </div>
                <div class="action" on:click=move |_| toggle_loan_modal()>"Loan Shark"</div>
                <div class="action double-width" on:click=move |_| toggle_bets_modal()>"Place Bet"</div>
            </div>
            <div class="card-line">
                <div class="card card-two"></div>
                <div class="card card-three"></div>
                <div class="card card-four"></div>
            </div>
        </div>
        <Show when=bets_modal fallback=||view!{}>
            <div class="pre-game-container">
                <div class="back-button" on:click=move |_| toggle_bets_modal()>"←"</div>
                <h2>"Place your Bets"</h2>
                <p>"Available = $"{available_money}</p>
                <For
                    each=move || bets
                    key=|it| *it
                    children=move |Bet { name, amount, .. }| {
                        view! {
                            <CreatureCard name amount available_money />
                        }
                    }
                />
                <div class="action confirm-bets" on:click=move |_| place_bets.dispatch(())>
                    "Confirm bets"
                </div>
            </div>
        </Show>
        <Show when=loan_modal fallback=|| view!{}>
            <LoanModal debt=debt.into_signal() account_balance close=toggle_loan_modal  />
        </Show>
    }
}

#[component]
fn loan_modal(
    close: impl Fn() + Copy + 'static,
    debt: Signal<u32>,
    account_balance: Signal<i32>,
) -> impl IntoView {
    let game_id = use_game_id();

    let minimum = move || -1 * i32::min(account_balance(), debt() as i32);

    let (borrow, set_borrow) = {
        let (read, write) = create_signal(0);

        (read, move |amount: i32| {
            write(i32::clamp(amount, minimum(), 1000));
        })
    };

    let increment = move |_: MouseEvent| set_borrow(borrow() + 100);
    let decrement = move |_: MouseEvent| set_borrow(borrow() - 100);

    let total_debt = move || debt() as i32 + borrow();

    let set_borrow_from_input = move |ev| {
        set_borrow(event_target_value(&ev).parse().unwrap_or_default());
    };

    let borrow_money = create_action(move |amount: &i32| {
        let amount = *amount;

        async move {
            match server_fn::<BorrowMoney>(game_id, &borrow_money::Input { amount }).await {
                Ok(_) => close(),
                Err(err) => tracing::error!(?err, "failed to borrow money"),
            };
        }
    });

    view! {
        <div class="pre-game-container">
            <div class="back-button" on:click=move |_| close()>"←"</div>
            <h1>"Loan shark"</h1>
            <div class="loan-shark"><p class="emoji">"🦈"</p></div>
            <p class="bio">"\"I'm a shark, How much do you want to borrow?\""</p>
            <p>"Interest rate = 5.1%"</p>
            <div class="creature-container">
                <p style="text-align: center;">"Current debt = $"{debt}{move || (debt() == 1000).then(|| view! { "(max)"})}</p>
                <div class="betting-row">
                    <button
                        on:click=decrement
                        disabled=move || (borrow() <= minimum())
                    >
                        "-"
                    </button>
                    <input
                        type="number"
                        prop:value=borrow
                        on:input=set_borrow_from_input
                    />
                    <button
                        on:click=increment
                        disabled=move || (total_debt() >= 1000)
                    >
                        "+"
                    </button>
                </div>
            </div>
            <div class="action confirm-bets" on:click=move |_| borrow_money.dispatch(borrow())>
                {
                    move ||
                        if borrow() >= 0 {
                            view!{ "Borrow" }
                        } else {
                            view! { "Payback" }
                        }
                }
            </div>
        </div>
    }
}
