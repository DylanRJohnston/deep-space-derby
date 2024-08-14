use std::cmp::{max, min};

use ev::MouseEvent;
use leptos::*;
use leptos_use::use_scroll;
use uuid::Uuid;

use crate::{
    server_fns::server_fn,
    utils::{use_events, use_game_id, use_session_id},
};
use shared::models::{
    cards::{Card, Target, TargetKind},
    commands::{borrow_money, place_bets, play_card, BorrowMoney, BuyCard, PlaceBets, PlayCard},
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
                <button on:click=decrement disabled=move || (amount() <= 0)>
                    "-"
                // {increment_size}
                </button>
                <input type="number" prop:value=amount on:input=arbitrary_amount/>
                <button on:click=increment disabled=move || (available_money() <= 0)>
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
    let monsters =
        projections::monsters(&untracked_events, projections::race_seed(&untracked_events));

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
            monster_id: uuid,
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

    let (card_modal, toggle_card_modal) = {
        let (read, write) = create_signal(false);

        (read, move || write(!read()))
    };

    let cards = move || projections::cards_in_hand(&events(), player_id);

    let buy_card = create_action(move |input| server_fn::<BuyCard>(game_id, input));
    let cards_disabled =
        Signal::derive(move || projections::already_played_card_this_round(&events(), player_id));

    view! {
        <div class="pre-game-container">
            // <div class="profile-image">"Profile Image"</div>
            <div class="player-info">
                <h2>{player_name}</h2>
                <div class="finance">
                    <span style="justify-self: end">"Funds:"</span>
                    <span>"💎 " {available_money}</span>
                    <span style="justify-self: end">"Debt:"</span>
                    <span>"💎 " {debt}</span>
                </div>
            </div>
            <div class="action-grid">
                <div class="placeholder-image">
                    <img src="/pkg/icons/spade.svg"/>
                </div>
                <div class="placeholder-image">
                    <img src="/pkg/icons/shark.svg"/>
                </div>
                <button
                    class="action"
                    on:click=move |_| buy_card.dispatch(())
                    disabled=move || (cards().len() >= 5)
                >
                    <p>"Buy a card"</p>
                    <p>"(💎 100)"</p>
                </button>
                <button class="action" on:click=move |_| toggle_loan_modal()>
                    "Loan Shark"
                </button>
                <button class="action double-width" on:click=move |_| toggle_bets_modal()>
                    "Place Bet"
                </button>
            </div>
            <div class="card-line">
                {move || {
                    let cards = cards();
                    let count = cards.len() as i32;
                    cards
                        .into_iter()
                        .enumerate()
                        .map(|(index, card)| {
                            let rotation = if count % 2 == 0 {
                                (index as i32 - count / 2) * 25 + 13
                            } else {
                                (index as i32 - count / 2) * 25
                            };
                            view! {
                                <CardPreview
                                    card
                                    rotation
                                    on_click=toggle_card_modal
                                    disabled=cards_disabled
                                />
                            }
                        })
                        .collect::<Vec<_>>()
                }}

            </div>
        </div>
        <Show when=bets_modal fallback=|| view! {}>
            <div class="pre-game-container blurred">
                <button class="back-button" on:click=move |_| toggle_bets_modal()>
                    "←"
                </button>
                // <h2>"Place your Bets"</h2>
                <p>"Available: 💎 " {available_money}</p>
                <For
                    each=move || bets
                    key=|it| *it
                    children=move |Bet { name, amount, .. }| {
                        view! { <CreatureCard name amount available_money/> }
                    }
                />

                <button class="action confirm-bets" on:click=move |_| place_bets.dispatch(())>
                    "Confirm bets"
                </button>
            </div>
        </Show>
        <Show when=loan_modal fallback=|| view! {}>
            <LoanModal debt=debt.into_signal() account_balance close=toggle_loan_modal/>
        </Show>
        <Show when=card_modal fallback=|| view! {}>
            <CardModal close=toggle_card_modal/>
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
        <div class="pre-game-container blurred">
            <button class="back-button" on:click=move |_| close()>
                "←"
            </button>
            // <h1>"Loan shark"</h1>
            <div class="loan-shark"></div>
            <p class="bio">"\"I'm a shark, How much do you want to borrow?\""</p>
            <p>"Interest Rate: 5.1%/pr"</p>
            <div class="creature-container">
                <p style="text-align: center;">
                    "Current debt:  💎 " {debt}
                    {move || (debt() == 1000).then(|| view! { "(max)" })}
                </p>
                <div class="betting-row">
                    <button on:click=decrement disabled=move || (borrow() <= minimum())>
                        "-"
                    </button>
                    <input type="number" prop:value=borrow on:input=set_borrow_from_input/>
                    <button on:click=increment disabled=move || (total_debt() >= 1000)>
                        "+"
                    </button>
                </div>
            </div>
            <button class="action confirm-bets" on:click=move |_| borrow_money.dispatch(borrow())>

                {move || {
                    if borrow() >= 0 {
                        view! { "Borrow" }
                    } else {
                        view! { "Payback" }
                    }
                }}

            </button>
        </div>
    }
}

#[component]
fn card_preview(
    card: Card,
    rotation: i32,
    on_click: impl Fn() + 'static,
    disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <button
            class="card"
            style=format!("transform: rotate({rotation}deg)")
            disabled=disabled
            on:click=move |_| on_click()
        >
            <p>{card.name()}</p>
            <img src=card.icon()/>
        </button>
    }
}

#[component]
fn card_main(card: Card) -> impl IntoView {
    view! {
        <div class="card-main">
            <p>{card.name()}</p>
            <img src=card.icon()/>
            <p>{card.description()}</p>
        </div>
    }
}

#[component]
fn card_modal(close: impl Fn() + Copy + 'static) -> impl IntoView {
    let events = use_events();
    let player_id = use_session_id();
    let cards = move || projections::cards_in_hand(&events(), player_id);

    let scroll_ref = create_node_ref::<leptos::html::Div>();

    let scroll = use_scroll(scroll_ref);

    let (target_modal, toggle_target_modal) = {
        let (read, write) = create_signal(false);

        (read, move || write(!read()))
    };

    let card = Signal::derive(move || {
        let index = (((scroll.x)() as f32 - 125.0) / 250.) as usize;
        cards().get(index).copied()
    });

    view! {
        <div class="pre-game-container blurred">
            <button class="back-button" on:click=move |_| close()>
                "←"
            </button>
            <div ref=scroll_ref class="card-carousel">
                {move || {
                    cards().into_iter().map(|card| view! { <CardMain card/> }).collect::<Vec<_>>()
                }}

            </div>
            <button
                class="action"
                disabled=scroll.is_scrolling
                on:click=move |_| toggle_target_modal()
            >
                "Play Card"
            </button>
        </div>
        {move || match card() {
            Some(card) if target_modal() => {
                view! { <TargetModal card close=toggle_target_modal done=close/> }.into_view()
            }
            _ => ().into_view(),
        }}
    }
}

#[component]
fn target_modal(
    card: Card,
    close: impl Fn() + Copy + 'static,
    done: impl Fn() + Copy + 'static,
) -> impl IntoView {
    let events = use_events();
    let game_id = use_game_id();
    let race_seed = move || projections::race_seed(&events());

    let (target, set_target) = create_signal::<Option<Uuid>>(None);

    let targets = move || match card.target_kind() {
        TargetKind::Monster => projections::monsters(&events(), race_seed())
            .into_iter()
            .map(|monster| {
                view! {
                    <button
                        class="card-target"
                        class:selected-target=move || target().unwrap_or_default() == monster.uuid
                        on:click=move |_| set_target(Some(monster.uuid))
                    >
                        <p>{monster.name}</p>
                    </button>
                }
                .into_view()
            })
            .collect::<Vec<View>>(),
        TargetKind::Player => projections::players(&events())
            .into_iter()
            .map(|(_, player)| {
                view! {
                    <button
                        class="card-target"
                        class:selected-target=move || {
                            target().unwrap_or_default() == player.session_id
                        }

                        on:click=move |_| set_target(Some(player.session_id))
                    >

                        <p>{player.name}</p>
                    </button>
                }
                .into_view()
            })
            .collect::<Vec<View>>(),
    };

    let play_card = create_action(move |_: &()| async move {
        let Some(target) = target() else {
            return;
        };

        let target = match card.target_kind() {
            TargetKind::Monster => Target::Monster(target),
            TargetKind::Player => Target::Player(target),
        };

        match server_fn::<PlayCard>(game_id, &play_card::Input { card, target }).await {
            Ok(_) => done(),
            Err(err) => tracing::error!(?err, "failed to play card"),
        };
    });

    view! {
        <div class="pre-game-container blurred">
            <button class="back-button" on:click=move |_| close()>
                "←"
            </button>
            <div class="card-target-container">
                <p>{card.description()}</p>
                {targets}
                <button
                    class="action target-confirm"
                    disabled=move || target().is_none()
                    on:click=move |_| play_card.dispatch(())
                >
                    "Confirm"
                </button>
            </div>
        </div>
    }
}
