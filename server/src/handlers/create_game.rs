use std::ops::Deref;

use crate::utils::err_wrapper::ErrWrapper;
use axum::{
    extract::{Request, State},
    http::{header::HeaderMap, Uri},
};
use shared::models::{
    commands::{self, Command},
    game_id::generate_game_code,
};
use worker::{Env, HttpResponse};

#[tracing::instrument(skip_all, err)]
#[axum::debug_handler]
#[worker::send]
pub async fn create_game(
    State(env): State<Env>,
    headers: HeaderMap,
    uri: Uri,
    _req: Request,
) -> Result<HttpResponse, ErrWrapper> {
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

    let req = do_req.body(serde_json::to_string(&commands::create_game::Input {
        code: game_code,
    })?)?;

    let response = env
        .durable_object("GAME")?
        .id_from_name(game_code.deref())?
        .get_stub()?
        .fetch_with_request(req.try_into()?)
        .await?;

    tracing::info!(status_code = ?response.status_code(), "response from durable object");

    if response.status_code() != 200 {
        return Ok(response.try_into()?);
    }

    let url = format!(
        "https://{host}{path}",
        host = uri.authority().unwrap(),
        path = commands::CreateGame::redirect(game_code).unwrap()
    );

    let response = web_sys::Response::redirect_with_status(&url, 303).map_err(|err| {
        tracing::error!(?err, ?url, "failed to construct redirect response");
        err
    })?;

    Ok(worker::response_from_wasm(response)?)
}
