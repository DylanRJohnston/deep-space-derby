mod player_lobby;
pub use player_lobby::*;

mod host_lobby;
pub use host_lobby::*;

mod game_wrapper;
pub use game_wrapper::*;

mod join_screen;
pub use join_screen::*;

mod main_menu;
pub use main_menu::*;

use leptos::{component, view};
use leptos_meta::{provide_meta_context, Stylesheet};
use leptos_router::{Route, Router, Routes};

#[component]
pub fn app() -> impl leptos::IntoView {
    provide_meta_context();

    view! {
        <Stylesheet href="/pkg/style.css"/>

        <Router>
            <Routes>
                <Route path="/" view=MainMenu/>
                <Route path="/host/:game_id" view=GameConnectionWrapper>
                    <Route path="/lobby" view=HostLobby/>
                </Route>
                <Route path="/play" view=JoinScreen/>
                <Route path="/play/:game_id" view=GameConnectionWrapper>
                    <Route path="/lobby" view=PlayerLobby />
                </Route>
            </Routes>
        </Router>
    }
}
