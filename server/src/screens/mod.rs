use leptos::{component, view, IntoView};
use leptos_meta::{provide_meta_context, Stylesheet};
use leptos_reactive::create_memo;

use crate::{models::events::Event, utils::use_events};

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
        <Router/>
    }
}

