use std::ops::Deref;

use crate::utils::err_wrapper::ErrWrapper;
use axum::{
    extract::{Request, State},
    http::header::HeaderMap,
};
use shared::models::{
    commands::{self, Command},
    game_id::generate_game_code,
};
use worker::{Env, HttpResponse};

#[tracing::instrument(skip_all)]
#[axum::debug_handler]
#[worker::send]
pub async fn create_game(
    State(env): State<Env>,
    headers: HeaderMap,
    _req: Request,
) -> Result<HttpResponse, ErrWrapper> {
    let game_code = generate_game_code();

    let mut req = Request::post(format!(
        "https://DURABLE_OBJECT{}",
        commands::CreateGame::url(game_code)
    ));

    *req.headers_mut().unwrap() = headers;
    req.headers_mut()
        .unwrap()
        .insert("Content-Type", "application/json".parse().unwrap());

    let req = req.body(serde_json::to_string(&commands::create_game::Input {
        code: game_code,
    })?)?;

    let response = env
        .durable_object("GAME")?
        .id_from_name(game_code.deref())?
        .get_stub()?
        .fetch_with_request(req.try_into()?)
        .await?;

    if response.status_code() != 200 {
        return Ok(response.try_into()?);
    }

    let url = commands::CreateGame::redirect(game_code).unwrap();
    let response = web_sys::Response::redirect_with_status(&url, 303)?;

    Ok(worker::response_from_wasm(response)?)
}
