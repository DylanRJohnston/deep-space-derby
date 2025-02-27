#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
pub async fn main() {
    use std::net::SocketAddr;

    use app::{
        adapters::{
            game_service::axum_router::AxumGameService, game_state::file::FileGameDirectory,
        },
        router::{into_game_router, into_outer_router},
    };
    use axum_server::tls_rustls::RustlsConfig;
    use tower_http::trace::{DefaultMakeSpan, TraceLayer};

    tracing_subscriber::fmt().pretty().init();

    let app = into_outer_router(AxumGameService {
        router: into_game_router(FileGameDirectory::default()),
    })
    .layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );

    let config =
        RustlsConfig::from_pem_file("self_signed_certs/cert.pem", "self_signed_certs/key.pem")
            .await
            .unwrap();

    let ssl_fut = axum_server::bind_rustls(SocketAddr::from(([0, 0, 0, 0], 8788)), config)
        .serve(app.clone().into_make_service());

    let fut = axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap(),
        app,
    );

    tokio::try_join!(ssl_fut, fut).unwrap();
}

#[cfg(target_arch = "wasm32")]
pub fn main() {}
