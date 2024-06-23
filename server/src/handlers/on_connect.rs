use axum::{body::Body, extract::State, response::Response};
use tracing::instrument;
use worker::WebSocketPair;

use crate::{
    durable_objects::game::GameWrapper, session_id::SessionID, utils::err_wrapper::ErrWrapper,
};

#[axum::debug_handler]
#[worker::send]
#[instrument(skip_all, err)]
pub async fn on_connect(
    State(mut game): State<GameWrapper>,
    SessionID(_session_id): SessionID,
) -> Result<Response, ErrWrapper> {
    let pair = WebSocketPair::new()?;

    // let metadata = Metadata { session_id };
    // pair.server.serialize_attachment(&metadata)?;

    game.state().accept_web_socket(&pair.server);

    for event in game.events().iter().await? {
        pair.server.send(event)?;
    }

    Ok(Response::builder()
        .status(101)
        .extension(pair.client)
        .body(Body::empty())?)
}
