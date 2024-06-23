use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub struct SessionID(pub Uuid);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for SessionID {
    type Rejection = &'static str;

    #[instrument(skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();

        jar.get("session_id")
            .ok_or("missing session_id cookie")
            .map(|it| it.value())
            .and_then(|it| Uuid::parse_str(it).map_err(|_| "Unable to parse session_id"))
            .map(SessionID)
    }
}
