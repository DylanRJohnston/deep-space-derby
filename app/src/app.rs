use leptos::{component, view, IntoView};
use leptos_meta::Meta;
use leptos_meta::{provide_meta_context, Stylesheet};

use crate::screens::router::Router;

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
        <Meta
            name="viewport"
            content="width=device-width, initial-scale=1, maximum-scale=1, minimum-scale=1, user-scalable=no"
        />
        <div class="background-image"></div>
        <div class="root-container" on:click=|_| {}>
            <Router/>
        </div>
    }
}
