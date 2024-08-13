use cookie::Cookie;
use leptos::{provide_context, use_context};
use shared::models::events::Event;
use uuid::Uuid;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast};

#[cfg(feature = "ssr")]
pub mod err_wrapper;
pub mod use_game_id;
pub mod use_websocket;

pub use use_game_id::*;
pub use use_websocket::*;

pub fn get_session_id() -> Option<Uuid> {
    let cookie_str = web_sys::window()?
        .document()?
        .dyn_into::<web_sys::HtmlDocument>()
        .ok()?
        .cookie()
        .ok()?;

    let cookie = Cookie::split_parse(&cookie_str)
        .filter_map(|it| it.ok())
        .find(|it| it.name() == "session_id")?;

    Uuid::parse_str(cookie.value()).ok()
}

#[derive(Clone, Copy)]
struct SessionID(Uuid);

pub fn provide_session_id(session_id: Uuid) {
    provide_context(SessionID(session_id))
}

pub fn use_session_id() -> Uuid {
    use_context::<SessionID>().unwrap().0
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "sendGameEvent")]
    pub fn send_game_event(event: Event);

    #[wasm_bindgen(js_name = "resetGameEvents")]
    pub fn reset_game_events();
}
