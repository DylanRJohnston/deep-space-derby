#![feature(type_alias_impl_trait)]
#![feature(try_blocks)]
#![feature(async_closure)]
#![feature(impl_trait_in_assoc_type)]
#![feature(impl_trait_in_fn_trait_return)]

use leptos::mount_to_body;
use wasm_bindgen::prelude::wasm_bindgen;

mod durable_objects;
mod models;
mod screens;
mod server_fns;
mod utils;

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();

    mount_to_body(screens::App);
}
