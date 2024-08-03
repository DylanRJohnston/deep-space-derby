use std::net::SocketAddr;

use app::{
    adapters::game_state::in_memory::InMemoryGameState,
    router::{into_game_router, into_outer_router},
    service::axum_router::AxumGameService,
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt().pretty().init();

    let app = into_outer_router(AxumGameService {
        router: into_game_router(InMemoryGameState::default()),
    })
    .layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );

    let config =
        RustlsConfig::from_pem_file("self_signed_certs/cert.pem", "self_signed_certs/key.pem")
            .await
            .unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
