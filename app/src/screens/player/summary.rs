use leptos::*;
use shared::models::projections;

use crate::utils::{use_events, use_session_id};

#[component]
pub fn summary() -> impl IntoView {
    let events = use_events();
    let player_id = use_session_id();

    let balance = move || {
        projections::all_account_balances(&events())
            .get(&player_id)
            .copied()
            .unwrap_or_default()
    };

    let debt = Memo::new(move |_| projections::debt(&events(), player_id));

    let winnings = Memo::new(move |_| {
        projections::winnings(&events())
            .get(&player_id)
            .copied()
            .unwrap_or_default()
    });

    let symbol = move || if winnings() >= 0 { "+" } else { "-" };
    let image = move || if winnings() >= 0 { "📈" } else { "📉" };

    view! {
        <div class="pre-game-container justify-center">
            <div class="payout-info">
                <h1>"Payout"</h1>
                <div class="payout-image">{image}</div>
                <div class="payout-amount">{symbol} "  💎" {move || winnings().abs()}</div>
                <div class="payout-table">
                    <div>"Funds:"</div>
                    <div>"💎 " {balance}</div>
                    <div>"Debt:"</div>
                    <div>"💎 " {debt}</div>
                    <div>"Score:"</div>
                    <div>"💎 " {move || balance() - (debt() as i32)}</div>
                </div>
            </div>
        </div>
    }
}
