use std::any::type_name;

use crate::{
    extractors::{Game, SessionID},
    ports::game_state::{GameDirectory, GameState},
};
use axum::{
    Json,
    response::{IntoResponse, Response},
    routing::post,
};
use shared::models::{
    commands::{API, CommandHandler},
    process_managers::run_processors,
};

use crate::ports::game_service::{GameService, InternalServerError};

pub trait RegisterCommandExt {
    fn register_command_handler<C: CommandHandler + API + 'static>(self) -> Self;
}

#[tracing::instrument(skip_all, fields(session_id, input, command = type_name::<C>()), err)]
pub async fn command_handler<C: CommandHandler, G: GameDirectory>(
    SessionID(session_id): SessionID,
    Game(game): Game<G>,
    Json(input): Json<C::Input>,
) -> Result<Response, InternalServerError> {
    let mut events = game.events().await?;

    let new_events = C::handle(session_id, &events, input)?;
    for event in new_events {
        game.push_event(event.clone()).await?;
        events.push_back(event);
    }

    let (new_events, alarm) = run_processors(&events)?;

    for event in new_events {
        game.push_event(event).await?;
    }

    if let Some(alarm) = alarm {
        game.set_alarm(alarm.0).await?;
    }

    Ok(().into_response())
}

impl<G: GameDirectory> RegisterCommandExt for axum::Router<G> {
    fn register_command_handler<C: CommandHandler + API + 'static>(self) -> Self {
        self.route(&C::url(":code"), post(command_handler::<C, G>))
    }
}
