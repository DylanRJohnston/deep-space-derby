use std::any::type_name;

use crate::{ports::game_state::GameState, service::InternalServerError, session_id::SessionID};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json,
};
use shared::models::commands::{Command, Effect};

pub trait CommandHandler {
    fn register_command<C: Command + 'static>(self) -> Self;
}

#[tracing::instrument(skip_all, fields(session_id, input, command = type_name::<C>()), err)]
pub async fn command_handler<C: Command, G: GameState>(
    SessionID(session_id): SessionID,
    State(game): State<G>,
    Json(input): Json<C::Input>,
) -> Result<Response, InternalServerError> {
    let (new_events, effect) = C::handle(session_id, &game.events().await?, input)?;
    for event in new_events {
        game.push_event(event).await?;
    }

    match effect {
        Some(Effect::SoftCommand(f)) => {
            if let Some(event) = f(&game.events().await?) {
                game.push_event(event).await?
            }
        }
        None => {}
        Some(Effect::Alarm(_)) => tracing::error!("alarms not currently implemented"),
    }

    Ok(().into_response())
}

impl<G: GameState> CommandHandler for axum::Router<G> {
    fn register_command<C: Command + 'static>(self) -> Self {
        self.route(&C::url(":code"), post(command_handler::<C, G>))
    }
}
