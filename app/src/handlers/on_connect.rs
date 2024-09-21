#[cfg(target_arch = "wasm32")]
mod wasm {
    use axum::response::Response;
    use shared::models::events::EventStream;
    use worker::WebSocketPair;

    use crate::{
        extractors::{Game, GameCode, SessionID},
        ports::game_state::{GameDirectory, GameState},
    };

    use crate::ports::game_service::{GameService, InternalServerError};

    pub type WebSocket = worker::WebSocket;

    // #[instrument(skip_all, err)]
    pub async fn on_connect<G: GameDirectory<WebSocket = WebSocket>>(
        Game(game_state): Game<G>,
        SessionID(_session_id): SessionID,
        GameCode { code }: GameCode,
    ) -> Result<Response, InternalServerError> {
        let pair = WebSocketPair::new()?;

        game_state.accept_web_socket(pair.server.clone()).await?;

        let events = EventStream::Events(game_state.events().await?.into_iter().collect());
        pair.server.send(&events)?;

        let response = Response::builder()
            .status(101)
            .extension(pair.client)
            .body(axum::body::Body::empty());

        Ok(response?)
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use axum::response::{IntoResponse, Response};
    use shared::models::events::EventStream;
    use tracing::instrument;

    use crate::ports::game_service::InternalServerError;
    use crate::{
        extractors::Game,
        ports::game_state::{GameDirectory, GameState},
    };

    pub type WebSocket = axum::extract::ws::WebSocket;

    #[instrument(skip_all, err)]
    pub async fn on_connect<G: GameDirectory<WebSocket = WebSocket>>(
        ws: axum::extract::WebSocketUpgrade,
        Game(game_state): Game<G>,
    ) -> Result<Response, InternalServerError> {
        Ok(ws
            .on_upgrade(move |mut ws| async move {
                let result: anyhow::Result<()> = try {
                    let events =
                        EventStream::Events(game_state.events().await?.into_iter().collect());

                    tracing::info!(?events);

                    let message = serde_json::to_string(&events)?;
                    ws.send(message.into()).await?;

                    game_state.accept_web_socket(ws).await?;
                };

                if let Err(err) = result {
                    tracing::error!(?err, "error upgrading websocket")
                }
            })
            .into_response())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
