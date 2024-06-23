use crate::utils::err_wrapper::ErrWrapper;
use axum::extract::{Path, Request, State};
use worker::{Env, HttpResponse};

#[axum::debug_handler]
#[worker::send]
#[tracing::instrument(skip_all, err)]
pub async fn forward_command(
    State(env): State<Env>,
    Path((code, _)): Path<(String, String)>,
    req: Request,
) -> Result<HttpResponse, ErrWrapper> {
    Ok(env
        .durable_object("GAME")?
        .id_from_name(&code)?
        .get_stub()?
        .fetch_with_request(req.try_into()?)
        .await?
        .try_into()?)
}
