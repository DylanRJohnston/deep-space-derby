use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use tracing::instrument;

use crate::{ports::game_state::GameState, service::InternalServerError, session_id::SessionID};

#[cfg(target_arch = "wasm32")]
#[instrument(skip_all, err)]
pub async fn on_connect<G: GameState>(
    State(game_state): State<G>,
    SessionID(_session_id): SessionID,
) -> Result<Response, InternalServerError> {
    let pair = game_state.accept_web_socket()?;

    for event in game_state.events().await?.into_iter() {
        pair.server.send(&event)?;
    }

    let response = Response::builder()
        .status(101)
        .extension(pair.client)
        .body(axum::body::Body::empty());

    Ok(response?)
}

#[cfg(not(target_arch = "wasm32"))]
#[instrument(skip_all, err)]
pub async fn on_connect<G: GameState>(
    ws: axum::extract::WebSocketUpgrade,
    State(game_state): State<G>,
    SessionID(_session_id): SessionID,
) -> Result<Response, InternalServerError> {
    Ok(ws
        .on_upgrade(|mut ws| async move {
            let result: anyhow::Result<()> = try {
                for event in game_state.events().await?.into_iter() {
                    let message = serde_json::to_string(&event)?;
                    ws.send(message.into()).await?;
                }

                game_state.accept_web_socket(ws).await?;
            };

            if let Err(err) = result {
                tracing::error!(?err, "error upgrading websocket")
            }
        })
        .into_response())
}
