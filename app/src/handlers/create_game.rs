use axum::{
    extract::{OriginalUri, Request, State},
    http::header::HeaderMap,
    response::Response,
};
use shared::models::{
    commands::{self, Command},
    game_id::generate_game_code,
};

use crate::service::{GameService, InternalServerError};

#[tracing::instrument(skip_all, err)]
pub async fn create_game<G: GameService>(
    State(mut game_service): State<G>,
    headers: HeaderMap,
    // request: Request,
) -> Result<Response, InternalServerError> {
    let game_code = generate_game_code();

    let mut do_req = Request::post(format!(
        "https://DURABLE_OBJECT{}",
        commands::CreateGame::url(game_code)
    ));

    *do_req.headers_mut().unwrap() = headers;
    do_req
        .headers_mut()
        .unwrap()
        .insert("Content-Type", "application/json".parse().unwrap());

    let req = do_req
        .body(serde_json::to_string(&commands::create_game::Input { code: game_code })?.into())?;

    let response = game_service.call((game_code.to_string(), req)).await?;

    if response.status() != 200 {
        let err = anyhow::anyhow!("non-200 received from game-service");
        tracing::error!(status = ?response.status(), ?err);
        Err(err)?;
    }

    let response = Response::builder()
        .header(
            "Location",
            commands::CreateGame::redirect(game_code).unwrap(),
        )
        .status(303)
        .body("".into())?;

    Ok(response)
}