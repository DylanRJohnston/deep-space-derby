use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use http::StatusCode;
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

pub trait GameService = where
    Self: Clone + Send + Sync + 'static,
    Self: Service<(String, Request), Response = Response, Error = InternalServerError>,
    <Self as Service<(String, Request)>>::Future: Send + 'static;
