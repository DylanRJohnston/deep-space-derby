use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use shared::models::game_code::GameCode;
use tower::Service;

#[derive(Debug)]
pub struct InternalServerError(anyhow::Error);

impl std::fmt::Display for InternalServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl IntoResponse for InternalServerError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl<E> From<E> for InternalServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub struct GameRequest {
    pub by: GameBy,
    pub req: Request,
}

pub enum GameBy {
    ID(String),
    Code(GameCode),
}

pub trait GameService = where
    Self: Clone + Send + Sync + 'static,
    Self: Service<GameRequest, Response = Response, Error = InternalServerError>,
    <Self as Service<GameRequest>>::Future: Send + 'static;
