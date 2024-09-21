use std::{future::Future, task::Poll};

use axum::{extract::Request, response::Response, Router};
use tower::Service;

use crate::ports::game_service::InternalServerError;

#[derive(Clone)]
pub struct AxumGameService {
    pub router: Router,
}

impl Service<(String, Request)> for AxumGameService {
    type Response = Response;

    type Error = InternalServerError;

    type Future = impl Future<Output = Result<Self::Response, Self::Error>> + Send + 'static;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, (_game_id, req): (String, Request)) -> Self::Future {
        let router = self.router.clone();

        async move { Ok(router.clone().call(req).await?) }
    }
}
