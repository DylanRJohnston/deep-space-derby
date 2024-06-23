#![feature(try_blocks)]
#![feature(async_closure)]
#![feature(impl_trait_in_assoc_type)]
#![feature(impl_trait_in_fn_trait_return)]
#![feature(more_qualified_paths)]

use std::{net::SocketAddr, str::FromStr};

use axum::http;
use axum::routing::{any, post};
use client::{
    handlers::{create_game::create_game, forward_command::forward_command, join_game::join_game},
    middleware::session_middleware,
    screens,
};
use leptos::{leptos_config, LeptosOptions};
use leptos_axum::{generate_route_list, LeptosRoutes};
use tower::Service;
use tracing::instrument;
use tracing_subscriber_wasm::MakeConsoleWriter;
use worker::{event, Env};

#[event(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    tracing_subscriber::fmt()
        .pretty()
        .with_writer(MakeConsoleWriter::default())
        .without_time()
        .init()
}

#[event(fetch)]
#[instrument(skip_all)]
pub async fn fetch(
    req: worker::HttpRequest,
    env: Env,
    _ctx: worker::Context,
) -> worker::Result<http::Response<axum::body::Body>> {
    let leptos_options = LeptosOptions::builder()
        .output_name("index")
        .site_pkg_dir("pkg")
        .env(leptos_config::Env::DEV)
        .site_addr(SocketAddr::from_str("127.0.0.1:8788").unwrap())
        .reload_port(3001)
        .build();

    let mut router = axum::Router::new()
        .route("/api/create_game", post(create_game))
        .route("/api/join_game", post(join_game))
        .route(
            "/api/object/game/by_code/:code/*command",
            any(forward_command),
        )
        .with_state(env)
        .layer(axum::middleware::from_fn(session_middleware))
        .leptos_routes(
            &leptos_options,
            generate_route_list(screens::App),
            screens::App,
        )
        .with_state(leptos_options);

    Ok(router.call(req).await?)
}

pub fn main() {}
