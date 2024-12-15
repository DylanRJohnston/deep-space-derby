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
    let race_seed = move || projections::race_seed(&events());

    let monsters = Memo::new(move |_| projections::monsters(&events(), race_seed()));
    let cards = Memo::new(move |_| projections::unique_played_monster_cards(&events()));

    let timer = leptos_use::use_interval(1000);

    let monster_cards = Memo::new(move |_| {
        tracing::info!("creating new signals for cards!!!!!!!!!!!!!!!!");
        monsters().map(|monster| (monster.uuid, create_signal::<Option<PlayedCardInfo>>(None)))
    });

    create_effect(move |_| {
        let counter = timer.counter.get();
        tracing::info!(?counter);

        let Some(card) = cards().get((counter / 4) as usize).copied() else {
            return;
        };

        let active = counter % 4 != 3 && counter != 0;

        for (id, (get_card, set_card)) in monster_cards() {
            if card.monster_id == id {
                set_card(Some(PlayedCardInfo {
                    active,
                    card: card.card,
                }));

                // (timer.pause)();
            } else {
                match get_card.get() {
                    Some(info) => set_card(Some(PlayedCardInfo {
                        active: false,
                        card: info.card,
                    })),
                    _ => {}
                }
            }
        }
    });

    let visible = move || ((timer.counter.get() / 4) as usize) < (cards().len());

    view! {
        <Show when=visible fallback=move || {}>
            <div class="host-race-container">
                <For each=monster_cards key=move |it| it.0 let:data>
                    <div
                        class="flip-card"
                        class:reveal=move || data.1.0.get().map(|it| it.active).unwrap_or_default()
                    >
                        <div class="flip-card-inner">
                            <div class="flip-card-front">
                                <img src="/pkg/icons/spade.svg"/>
                            </div>
                            <div class="flip-card-back">
                                <h1>{move || data.1.0.get().map(|card| card.card.name())}</h1>
                                <img src=move || data.1.0.get().map(|card| card.card.icon())/>
                                <p>{move || data.1.0.get().map(|card| card.card.description())}</p>
                            </div>
                        </div>
                    </div>
                </For>
            </div>
        </Show>
    }
}
