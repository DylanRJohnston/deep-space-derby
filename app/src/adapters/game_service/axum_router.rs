use std::{future::Future, task::Poll};

use axum::{response::Response, Router};
use tower::Service;

use crate::ports::game_service::{GameRequest, InternalServerError};

#[derive(Clone)]
pub struct AxumGameService {
    pub router: Router,
}

impl Service<GameRequest> for AxumGameService {
    type Response = Response;

    type Error = InternalServerError;

    type Future = impl Future<Output = Result<Self::Response, Self::Error>> + Send + 'static;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, GameRequest { req, .. }: GameRequest) -> Self::Future {
        let router = self.router.clone();

        async move { Ok(router.clone().call(req).await?) }
    }
}
