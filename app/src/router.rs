use std::{net::SocketAddr, str::FromStr};

use axum::{
    extract::Request,
    middleware::Next,
    routing::{any, get, post},
};
use http::Uri;
use leptos::{leptos_config, LeptosOptions};
use leptos_axum::{generate_route_list, LeptosRoutes};
use shared::models::commands;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use crate::{
    handlers::{
        create_game::create_game, forward_command::forward_command, join_game::join_game,
        on_connect::on_connect, register_command::CommandHandler,
    },
    middleware::session_middleware,
    ports::game_state::GameState,
    screens,
    service::GameService,
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
            any(forward_command::<S>),
        )
        .with_state(game_service)
        .layer(axum::middleware::from_fn(session_middleware))
        .leptos_routes(
            &leptos_options,
            generate_route_list(screens::App),
            screens::App,
        );

    #[cfg(not(target_arch = "wasm32"))]
    let router = router.fallback(crate::serve_files::file_and_error_handler);

    router.with_state(leptos_options)
}

pub fn into_game_router<G: GameState>(game: G) -> axum::Router {
    tracing::info!(
        game_state = std::any::type_name::<G>(),
        "constructing game router"
    );

    axum::Router::new()
        .route(
            "/api/object/game/by_code/:code/connect",
            get(on_connect::<G>),
        )
        .register_command::<commands::CreateGame>()
        .register_command::<commands::JoinGame>()
        .register_command::<commands::ChangeProfile>()
        .register_command::<commands::ReadyPlayer>()
        .register_command::<commands::PlaceBets>()
        .layer(axum::middleware::from_fn(
            |uri: Uri, request: Request, next: Next| async move {
                tracing::info!(?uri, method = ?request.method(), "game router received request");

                let response = next.run(request).await;

                tracing::info!(?response, "game router response");

                response
            },
        ))
        .with_state(game)
}
