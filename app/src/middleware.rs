use axum::{extract::Request, middleware::Next, response::Response};
use axum_extra::extract::CookieJar;
use cookie::CookieBuilder;
use uuid::Uuid;

use crate::extractors::SessionID;

#[tracing::instrument(skip_all)]
pub async fn session_middleware(
    session_id: Option<SessionID>,
    cookie_jar: CookieJar,
    mut request: Request,
    next: Next,
) -> (CookieJar, Response) {
    let mut cookie_jar = cookie_jar;

    let session_id = match session_id {
        Some(session_id) => session_id,
        None => {
            let session_id = SessionID(Uuid::new_v4());
            cookie_jar = cookie_jar.add(
                CookieBuilder::new("session_id", session_id.0.to_string())
                    .path("/")
                    .secure(true)
                    .http_only(false)
                    .same_site(cookie::SameSite::Strict)
                    .build(),
            );

            session_id
        }
    };

    request.extensions_mut().insert(session_id);

    (cookie_jar, next.run(request).await)
}
