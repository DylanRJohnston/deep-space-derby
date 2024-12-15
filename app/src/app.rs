use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet};

use crate::screens::router::Router;

const AUDIO_CONTEXT_SCRIPT: &str = include_str!("js/audio_context.js");
const EVENT_BUFFER_FLUSH_SCRIPT: &str = include_str!("js/event_buffer_flush.js");

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta
                    name="viewport"
                    content="width=device-width, initial-scale=1, maximum-scale=1, minimum-scale=1, user-scalable=no"
                />
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options.clone()/>
                <MetaTags/>
                <script>{EVENT_BUFFER_FLUSH_SCRIPT}</script>
                <script>{AUDIO_CONTEXT_SCRIPT}</script>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn app() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/style.css"/>
        <div class="background-image"></div>
        <div class="root-container" on:click=|_| {}>
            <Router/>
        </div>
    }
}
