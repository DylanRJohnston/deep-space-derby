use crate::service::{GameService, InternalServerError};
use axum::{
    extract::{Request, State},
    http::header::HeaderMap,
    response::Response,
    Form,
};
use shared::models::commands::{self, Command, JoinGame};

#[tracing::instrument(skip_all, err)]
pub async fn join_game<G: GameService>(
    State(mut game_service): State<G>,
    headers: HeaderMap,
    Form(join_game): Form<<JoinGame as Command>::Input>,
) -> Result<Response, InternalServerError> {
    let mut req = Request::post(format!(
        "https://DURABLE_OBJECT{}",
        JoinGame::url(join_game.code)
    ));

    *req.headers_mut().unwrap() = headers;
    req.headers_mut()
        .unwrap()
        .insert("Content-Type", "application/json".parse().unwrap());

    let req = req.body(serde_json::to_string(&join_game)?.into())?;

    let response = game_service.call((join_game.code.to_string(), req)).await?;

    if response.status() != 200 {
        tracing::error!(status = ?response.status(), "non-200 received from game");
        return Ok(Response::builder().status(500).body("".into())?);
    }

    let response = Response::builder()
        .header(
            "Location",
            commands::JoinGame::redirect(join_game.code).unwrap(),
        )
        .status(303)
        .body("".into())?;

    Ok(response)
}
