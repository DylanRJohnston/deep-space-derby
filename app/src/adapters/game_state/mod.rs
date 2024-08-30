#[cfg(target_arch = "wasm32")]
pub mod durable_object;
#[cfg(not(target_arch = "wasm32"))]
pub mod file;
