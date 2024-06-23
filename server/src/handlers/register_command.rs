use std::any::type_name;

use crate::{
    durable_objects::game::GameWrapper, session_id::SessionID, utils::err_wrapper::ErrWrapper,
};
use axum::{extract::State, routing::post, Json};
use shared::models::commands::{Command, Effect};

pub trait CommandHandler {
    fn register_command<C: Command + 'static>(self) -> Self;
}

#[worker::send]
#[tracing::instrument(skip_all, fields(session_id, input), err)]
pub async fn command_handler<C: Command>(
    SessionID(session_id): SessionID,
    State(mut game): State<GameWrapper>,
    Json(input): Json<C::Input>,
) -> Result<(), ErrWrapper> {
    let (new_events, effect) = C::handle(session_id, game.events().vector().await?, input)?;

    for event in new_events {
        game.add_event(event).await?;
    }

    match effect {
        Some(Effect::Alarm(time)) => {
            match game.state().storage().get_alarm().await? {
                Some(_) => {
                    tracing::warn!(
                        "{} attempted to set an alarm while one was already set, noop",
                        type_name::<C>()
                    )
                }
                None => game.state().storage().set_alarm(time).await?,
            };
        }
        Some(Effect::SoftCommand(f)) => {
            if let Some(event) = f(game.events().vector().await?) {
                game.add_event(event).await?
            }
        }
        None => {}
    }

    Ok(())
}

impl CommandHandler for axum::Router<GameWrapper> {
    fn register_command<C: Command + 'static>(self) -> Self {
        self.route(&C::url(":code"), post(command_handler::<C>))
    }
}
