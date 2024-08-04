#![feature(type_alias_impl_trait)]
#![feature(try_blocks)]
#![feature(async_closure)]
#![feature(impl_trait_in_assoc_type)]
#![feature(impl_trait_in_fn_trait_return)]
#![feature(trait_alias)]
#![feature(stmt_expr_attributes)]

pub mod components;
pub mod screens;
pub mod server_fns;
pub mod utils;

#[cfg(feature = "ssr")]
pub mod adapters;
#[cfg(feature = "ssr")]
pub mod extractors;
#[cfg(feature = "ssr")]
pub mod handlers;
#[cfg(feature = "ssr")]
pub mod middleware;
#[cfg(feature = "ssr")]
pub mod ports;
#[cfg(feature = "ssr")]
pub mod router;
#[cfg(all(feature = "ssr", not(target_arch = "wasm32")))]
pub mod serve_files;
#[cfg(feature = "ssr")]
pub mod service;
