use std::{net::SocketAddr, str::FromStr};

use axum::routing::{any, get, post};
use leptos::{leptos_config, LeptosOptions};
use leptos_axum::{generate_route_list, LeptosRoutes};
use shared::models::commands;

use crate::{
    app,
    handlers::{
        create_game::create_game,
        event_log::event_log,
        forward_command::{forward_command_by_code, forward_command_by_id},
        join_game::join_game,
        on_connect::{on_connect, WebSocket},
        register_command::RegisterCommandExt,
        wake_up::wake_up,
    },
    middleware::session_middleware,
    ports::{game_service::GameService, game_state::GameDirectory},
};

pub fn into_outer_router<S: GameService>(game_service: S) -> axum::Router {
    let leptos_options = LeptosOptions::builder()
        .output_name("index")
        .site_root("site")
        .site_pkg_dir("pkg")
        .env(leptos_config::Env::DEV)
        .site_addr(SocketAddr::from_str("127.0.0.1:8788").unwrap())
        .reload_port(3001)
        .build();

    let router = axum::Router::new()
        .route("/api/create_game", post(create_game::<S>))
        .route("/api/join_game", post(join_game::<S>))
        .route(
            "/api/object/game/by_code/:code/*command",
            any(forward_command_by_code::<S>),
        )
        .route(
            "/api/object/game/by_id/:game_id/*command",
            any(forward_command_by_id::<S>),
        )
        .with_state(game_service)
        .leptos_routes(&leptos_options, generate_route_list(app::App), app::App)
        .layer(axum::middleware::from_fn(session_middleware));

    #[cfg(not(target_arch = "wasm32"))]
    let router = router.fallback(crate::serve_files::file_and_error_handler);

    router.with_state(leptos_options)
}

pub fn into_game_router<G: GameDirectory<WebSocket = WebSocket>>(game: G) -> axum::Router {
    axum::Router::new()
        .route(
            "/api/object/game/by_code/:code/connect",
            get(on_connect::<G>),
        )
        .route("/api/object/game/by_code/:code/wake_up", post(wake_up::<G>))
        .route(
            "/api/object/game/by_code/:code/event_log",
            get(event_log::<G>),
        )
        .route(
            "/api/object/game/by_id/:game_id/event_log",
            get(event_log::<G>),
        )
        .register_command_handler::<commands::CreateGame>()
        .register_command_handler::<commands::JoinGame>()
        .register_command_handler::<commands::ChangeProfile>()
        .register_command_handler::<commands::ReadyPlayer>()
        .register_command_handler::<commands::PlaceBets>()
        .register_command_handler::<commands::BorrowMoney>()
        .register_command_handler::<commands::BuyCard>()
        .register_command_handler::<commands::PlayCard>()
        .with_state(game)
}
