use leptos::{component, view, IntoView};
use leptos_meta::{provide_meta_context, Stylesheet};

use router::Router;

pub mod game_wrapper;
pub mod host;
pub mod main_menu;
pub mod player;
pub mod router;

#[component]
pub fn app() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/style.css"/>
        <script>
            "
            let pendingEvents = [];
            function sendGameEvent(event) {
                if (typeof globalThis['innerSendGameEvent'] !== 'function') {
                    pendingEvents.push(event);
                } else {
                    globalThis['innerSendGameEvent'](event);
                }
            }
            
            function resetGameEvents() {
                if (typeof globalThis['innerResetGameEvents'] === 'function') {
                    globalThis['innerResetGameEvents']();
                }
            }
            "
        </script>
        <div style="position: fixed; width: 100vw; height: 100vh;">
            <Router/>
        </div>
    }
}
