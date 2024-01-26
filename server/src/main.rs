#![feature(try_blocks)]
#![feature(async_closure)]

use std::net::SocketAddr;

use leptos::*;
use leptos_cloudflare::{LeptosRoutes, WorkerRouterData};
use std::str::FromStr;
use utils::generate_game_code;
use worker::{event, Method, Request};

mod durable_objects;
mod models;
mod screens;
mod server_fns;
mod utils;

#[event(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[event(fetch)]
pub async fn fetch(
    req: worker::Request,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<worker::Response> {
    worker::Router::with_data(WorkerRouterData {
        options: LeptosOptions::builder()
            .output_name("index")
            .site_pkg_dir("pkg")
            .env(leptos_config::Env::DEV)
            .site_addr(SocketAddr::from_str("127.0.0.1:8788").unwrap())
            .reload_port(3001)
            .build(),
        app_fn: screens::App,
    })
    .leptos_routes(leptos_cloudflare::generate_route_list(screens::App))
    .post_async("/api/create_game", |_req, ctx| async move {
        let game_code = generate_game_code();

        let request = Request::new(
            &format!(
                "https://localhost/api/object/game/by_code/{}/command/create_game",
                game_code
            ),
            Method::Post,
        )?;

        ctx.durable_object("game")?
            .id_from_name(&game_code)?
            .get_stub()?
            .fetch_with_request(request)
            .await
    })
    .on_async(
        "/api/object/game/by_code/:code/*command",
        |req, ctx| async move {
            let object_name = ctx
                .param("code")
                .ok_or("failed to find game code parameter in route")?;

            ctx.durable_object("game")?
                .id_from_name(object_name)?
                .get_stub()?
                .fetch_with_request(req)
                .await
        },
    )
    .run(req, env)
    .await
}

pub fn main() {}
