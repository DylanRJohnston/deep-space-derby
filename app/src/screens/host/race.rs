use leptos::prelude::*;
use leptos_use::UseIntervalReturn;
use shared::models::{cards::Card, projections};

use crate::utils::use_events;

#[derive(Debug, Clone, Copy)]
struct PlayedCardInfo {
    active: bool,
    card: Card,
}

#[derive(Debug, Clone, Copy)]
enum Countdown {
    Counting(i64),
    Finished,
}

#[component]
pub fn race() -> impl IntoView {
    let events = use_events();
    let race_seed = projections::race::race_seed(&events.get_untracked());

    let monsters = projections::monsters(&events.get_untracked(), race_seed);
    let cards = projections::unique_played_monster_cards(&events.get_untracked());

    let UseIntervalReturn { counter: timer, .. } = leptos_use::use_interval(1000);

    let monster_cards =
        monsters.map(|monster| (monster.uuid, signal::<Option<PlayedCardInfo>>(None)));

    Effect::new({
        let cards = cards.clone();
        move |_| {
            let counter = timer.get();

            tracing::info!(?counter, "counter trigger");

            let Some(card) = cards.get((counter / 4) as usize).copied() else {
                return;
            };

            let active = counter % 4 != 3 && counter != 0;

            for (id, (get_card, set_card)) in monster_cards {
                if card.monster_id == id {
                    set_card(Some(PlayedCardInfo {
                        active,
                        card: card.card,
                    }));
                } else {
                    match get_card.get_untracked() {
                        Some(info) => set_card(Some(PlayedCardInfo {
                            active: false,
                            card: info.card,
                        })),
                        _ => {}
                    }
                }
            }
        }
    });

    let visible = {
        let cards = cards.clone();

        move || ((timer.get() / 4) as usize) < (cards.len())
    };

    let countdown = Signal::derive(move || {
        if ((timer.get() / 4) as usize) < (cards.len()) {
            return None;
        }

        let value = 3 + cards.len() as i64 * 4 - timer.get() as i64;

        match value {
            1.. => Some(Countdown::Counting(value)),
            0 => Some(Countdown::Finished),
            _ => None,
        }
    });

    let go = move || matches!(countdown(), Some(Countdown::Finished));

    view! {
        <Show when=visible.clone() fallback=move || {}>
            <div class="host-race-container">
                {monster_cards
                    .map(|data| {
                        view! {
                            <div
                                class="flip-card"
                                class:reveal=move || {
                                    data.1.0.get().map(|it| it.active).unwrap_or_default()
                                }
                            >
                                <div class="flip-card-inner">
                                    <div class="flip-card-front">
                                        <img src="/pkg/icons/spade.svg" />
                                    </div>
                                    <div class="flip-card-back">
                                        <h1>
                                            {move || data.1.0.get().map(|card| card.card.name())}
                                        </h1>
                                        <img src=move || {
                                            data.1.0.get().map(|card| card.card.icon())
                                        } />
                                        <p>
                                            {move || data.1.0.get().map(|card| card.card.description())}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        }
                    })}
            </div>
        </Show>
        <Show when=move || countdown().is_some()>
            <div class="host-race-container">
                <div class="glow-circle" class:red=go></div>
                <div class="pulse-ring" class:red=go></div>
                <div class="countdown-display" class:red=go>
                    {move || {
                        match countdown() {
                            Some(Countdown::Counting(value)) => value.to_string(),
                            Some(Countdown::Finished) => "Go!".to_string(),
                            None => "".to_string(),
                        }
                    }}
                </div>
            </div>
        </Show>
    }
}
