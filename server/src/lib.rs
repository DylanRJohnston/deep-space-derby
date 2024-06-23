#![feature(type_alias_impl_trait)]
#![feature(try_blocks)]
#![feature(async_closure)]
#![feature(impl_trait_in_assoc_type)]
#![feature(impl_trait_in_fn_trait_return)]

use leptos::mount_to_body;
use wasm_bindgen::prelude::wasm_bindgen;

pub mod components;
#[cfg(feature = "ssr")]
pub mod durable_objects;
#[cfg(feature = "ssr")]
pub mod handlers;
#[cfg(feature = "ssr")]
pub mod middleware;
pub mod screens;
pub mod server_fns;
#[cfg(feature = "ssr")]
pub mod session_id;
pub mod utils;

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();

    mount_to_body(screens::App);
}
