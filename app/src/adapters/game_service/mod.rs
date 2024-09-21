#[cfg(not(target_arch = "wasm32"))]
pub mod axum_router;

#[cfg(target_arch = "wasm32")]
pub mod durable_object;
