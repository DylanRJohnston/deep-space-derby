use axum::Json;
use im::Vector;
use shared::models::events::Event;

use crate::{
    extractors::Game,
    ports::{
        game_service::InternalServerError,
        game_state::{GameDirectory, GameState},
    },
};

pub async fn event_log<G: GameDirectory>(
    Game(game): Game<G>,
) -> Result<Json<Vector<Event>>, InternalServerError> {
    Ok(Json(game.events().await?))
}
