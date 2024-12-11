use std::{error::Error, future::Future, task::Poll};

use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use tower::Service;
use worker::{send::SendFuture, Stub};

use crate::ports::game_service::{GameBy, GameRequest, InternalServerError};

#[derive(Clone)]
pub struct DurableObjectGameService {
    pub env: worker::Env,
}

impl Service<GameRequest> for DurableObjectGameService {
    type Response = Response;

    type Error = InternalServerError;

    type Future = impl Future<Output = Result<Self::Response, Self::Error>> + Send + 'static;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, GameRequest { by, req }: GameRequest) -> Self::Future {
        let stub: worker::Result<Stub> = try {
            match by {
                GameBy::ID(id) => self
                    .env
                    .durable_object("GAME")?
                    .id_from_string(&id)?
                    .get_stub()?,
                GameBy::Code(code) => self
                    .env
                    .durable_object("GAME")?
                    .id_from_name(&code.to_string())?
                    .get_stub()?,
            }
        };

        SendFuture::new(async move {
            Ok(stub?
                .fetch_with_request(req.try_into()?)
                .await?
                .try_into()?)
        })
    }
}
