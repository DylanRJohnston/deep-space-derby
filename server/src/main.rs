#![feature(try_blocks)]
#![feature(async_closure)]
#![feature(impl_trait_in_assoc_type)]
#![feature(impl_trait_in_fn_trait_return)]
#![feature(more_qualified_paths)]

use std::net::SocketAddr;

use futures_util::Future;
use leptos::*;
use leptos_cloudflare::{LeptosRoutes, WorkerRouterData};
use shared::models::{
    commands::{Command, CreateGame, GameCode, JoinGame},
    game_id::generate_game_code,
};
use std::{convert::identity, str::FromStr};
use wasm_bindgen::JsValue;
use worker::{event, Method, Request, RequestInit, RouteContext};

mod components;
mod durable_objects;
mod screens;
mod server_fns;
mod utils;

#[event(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

pub fn redirect_to_durable_object<C, D, F, Fut>(
    extractor: F,
) -> impl Fn(Request, RouteContext<D>) -> impl Future<Output = worker::Result<worker::Response>>
where
    C: Command,
    <C as Command>::Input: GameCode,
    F: Copy + Fn(Request) -> Fut,
    Fut: Future<Output = worker::Result<C::Input>>,
{
    identity(move |req: Request, ctx: RouteContext<D>| async move {
        let headers = req.headers().clone();
        let input = extractor(req).await?;

        let request = Request::new_with_init(
            &format!("https://localhost{}", C::url(input.game_code())),
            RequestInit::new()
                .with_method(Method::Post)
                .with_headers(headers)
                .with_body(Some(JsValue::from_str(
                    &serde_qs::to_string(&input).map_err(|_| worker::Error::BadEncoding)?,
                ))),
        )?;

        let inner_response = ctx
            .durable_object("game")?
            .id_from_name(&input.game_code())?
            .get_stub()?
            .fetch_with_request(request)
            .await?;

        match C::redirect(input.game_code()) {
            None => Ok(inner_response),
            Some(location) => {
                if inner_response.status_code() != 200 {
                    return Ok(inner_response);
                }

                let mut headers = inner_response.headers().clone();
                headers.append("Location", &location)?;

                Ok(web_sys::Response::new_with_opt_str_and_init(
                    None,
                    web_sys::ResponseInit::new()
                        .status(302)
                        .headers(&headers.0.into()),
                )?
                .into())
            }
        }
    })
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
    .post_async(
        "/api/create_game",
        redirect_to_durable_object::<CreateGame, _, _, _>(|_| async {
            Ok(<CreateGame as Command>::Input {
                code: generate_game_code(),
            })
        }),
    )
    .post_async(
        "/api/join_game",
        redirect_to_durable_object::<JoinGame, _, _, _>(|mut req| async move {
            serde_qs::from_str::<<JoinGame as Command>::Input>(&req.text().await?)
                .map_err(|_| worker::Error::BadEncoding)
        }),
    )
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

