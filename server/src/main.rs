use std::net::SocketAddr;

use leptos::*;
use leptos_cloudflare::LeptosRoutes;
use std::str::FromStr;
use worker::{event, Response};

mod app;
mod durable_object;
mod leptos_cloudflare;
mod sessions;

#[event(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    app::HelloWorldFn::register_explicit().unwrap();
}

#[event(fetch)]
pub async fn fetch(
    req: worker::Request,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<worker::Response> {
    // let leptos_options = AppState {
    //     leptos_options: LeptosOptions::builder()
    //         .output_name("client")
    //         .site_pkg_dir("pkg")
    //         .build(),
    //     env: env.into(),
    // };

    // env.durable_object("Example")?
    //     .id_from_name("name")?
    //     .get_stub()?
    //     .fetch_with_request(req)
    //     .await

    worker::console_log!("Worker request: {}", req.path());

    worker::Router::with_data(leptos_cloudflare::WorkerRouterData {
        options: LeptosOptions::builder()
            .output_name("index")
            .site_pkg_dir("pkg")
            .env(leptos_config::Env::DEV)
            .site_addr(SocketAddr::from_str("127.0.0.1:3000").unwrap())
            .build(),
        app_fn: app::App,
    })
    .leptos_routes(leptos_cloudflare::generate_route_list(app::App))
    .post_async("/api/:fn_name", leptos_cloudflare::handle_server_fns)
    .get("/hello", |_, _| Response::ok("Hello, World"))
    .run(req, env)
    .await
}

pub fn main() {}
