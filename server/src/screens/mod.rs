use leptos::{component, view, IntoView};
use leptos_meta::{provide_meta_context, Stylesheet};

use router::Router;

mod game_wrapper;
mod host;
mod main_menu;
mod player;
mod router;

#[component]
pub fn app() -> impl leptos::IntoView {
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
            "
        </script>
        <div style="position: fixed; width: 100vw; height: 100vh;">
            <Router/>
        </div>
    }
}

