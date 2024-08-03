use app::screens;
use leptos::mount_to_body;
use wasm_bindgen::prelude::wasm_bindgen;

pub fn main() {}

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();

    mount_to_body(screens::App);
}
