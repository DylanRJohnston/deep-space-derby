use std::ops::Deref;

use crate::utils::err_wrapper::ErrWrapper;
use axum::{
    extract::{Request, State},
    http::{header::HeaderMap, Uri},
    Form,
};
use shared::models::commands::{self, Command, JoinGame};
use worker::{Env, HttpResponse};

#[tracing::instrument(skip_all, err)]
#[axum::debug_handler]
#[worker::send]
pub async fn join_game(
    State(env): State<Env>,
    headers: HeaderMap,
    uri: Uri,
    Form(join_game): Form<<JoinGame as Command>::Input>,
) -> Result<HttpResponse, ErrWrapper> {
    let mut req = Request::post(format!(
        "https://DURABLE_OBJECT{}",
        JoinGame::url(join_game.code)
    ));

    *req.headers_mut().unwrap() = headers;
    req.headers_mut()
        .unwrap()
        .insert("Content-Type", "application/json".parse().unwrap());

    let req = req.body(serde_json::to_string(&join_game)?)?;

    let response = env
        .durable_object("GAME")?
        .id_from_name(join_game.code.deref())?
        .get_stub()?
        .fetch_with_request(req.try_into()?)
        .await?;

    if response.status_code() != 200 {
        return Ok(response.try_into()?);
    }

    let url = format!(
        "https://{host}{path}",
        host = uri.authority().unwrap(),
        path = commands::JoinGame::redirect(join_game.code).unwrap()
    );

    let response = web_sys::Response::redirect_with_status(&url, 303).map_err(|err| {
        tracing::error!(?err, url, "failed to redirect to construct redirect");
        err
    })?;

    Ok(worker::response_from_wasm(response)?)
}
