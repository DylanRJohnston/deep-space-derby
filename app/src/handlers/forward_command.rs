use axum::{
    extract::{Path, Request, State},
    response::Response,
};

use crate::{
    extractors::{GameCode, GameID},
    ports::game_service::{GameBy, GameRequest, GameService, InternalServerError},
};

#[tracing::instrument(skip_all, err)]
pub async fn forward_command_by_code<G: GameService>(
    State(mut service): State<G>,
    Path(code): Path<GameCode>,
    req: Request,
) -> Result<Response, InternalServerError> {
    let response = service
        .call(GameRequest {
            by: GameBy::Code(code.code),
            req,
        })
        .await?;

    Ok(response)
}

#[tracing::instrument(skip_all, err)]
pub async fn forward_command_by_id<G: GameService>(
    State(mut service): State<G>,
    Path(GameID { game_id }): Path<GameID>,
    req: Request,
) -> Result<Response, InternalServerError> {
    let response = service
        .call(GameRequest {
            by: GameBy::ID(game_id),
            req,
        })
        .await?;

    Ok(response)
}
