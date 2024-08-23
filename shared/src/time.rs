#[cfg(not(feature = "wasm"))]
pub use std::time::*;

#[cfg(feature = "wasm")]
pub use web_time::*;
