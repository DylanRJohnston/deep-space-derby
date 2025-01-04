use leptos::prelude::*;
use shared::models::{cards::Card, projections};

use crate::utils::use_events;

#[derive(Debug, Clone, Copy)]
struct PlayedCardInfo {
    active: bool,
    card: Card,
}

#[component]
pub fn race() -> impl IntoView {
    let events = use_events();
    let race_seed = projections::race_seed(&events.get_untracked());

    let monsters = projections::monsters(&events.get_untracked(), race_seed);
    let cards = projections::unique_played_monster_cards(&events.get_untracked());

    let timer = leptos_use::use_interval(1000);

    let monster_cards =
        monsters.map(|monster| (monster.uuid, signal::<Option<PlayedCardInfo>>(None)));

    Effect::new({
        let cards = cards.clone();
        move |_| {
            let counter = timer.counter.get();

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

        move || ((timer.counter.get() / 4) as usize) < (cards.len())
    };

    view! {
        <Show when=visible fallback=move || {}>
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
    }
}
