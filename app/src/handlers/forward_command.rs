use axum::{
    extract::{Path, Request, State},
    response::Response,
};

use crate::service::{GameService, InternalServerError};

#[tracing::instrument(skip_all, err)]
pub async fn forward_command<G: GameService>(
    State(mut service): State<G>,
    Path((code, _)): Path<(String, String)>,
    req: Request,
) -> Result<Response, InternalServerError> {
    let response = service.call((code, req)).await?;

    Ok(response)
}
