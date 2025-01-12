use std::collections::HashMap;

use anyhow::anyhow;
use axum::{
    async_trait,
    extract::{FromRequestParts, Path},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use shared::models::game_code;
use tracing::instrument;
use uuid::Uuid;

use crate::ports::{
    game_service::{GameBy, InternalServerError},
    game_state::GameDirectory,
};

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

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct GameCode {
    pub code: game_code::GameCode,
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for GameCode {
    type Rejection = InternalServerError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Path(params) = parts.extract::<Path<HashMap<String, String>>>().await?;
        let game_code = params
            .get("code")
            .ok_or_else(|| anyhow!("failed to get game_id from path"))?
            .as_str()
            .try_into()?;

        Ok(GameCode { code: game_code })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GameID {
    pub game_id: String,
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for GameID {
    type Rejection = InternalServerError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Path(params) = parts.extract::<Path<HashMap<String, String>>>().await?;
        let game_id = params
            .get("game_id")
            .ok_or_else(|| anyhow!("failed to get game_id from path"))?
            .as_str();

        Ok(GameID {
            game_id: game_id.to_string(),
        })
    }
}

#[derive(Clone)]
pub struct Game<G: GameDirectory>(pub <G as GameDirectory>::GameState);

#[async_trait]
impl<G: GameDirectory> FromRequestParts<G> for Game<G> {
    type Rejection = InternalServerError;

    #[instrument(skip_all, err, fields(uri = ?parts.uri))]
    async fn from_request_parts(parts: &mut Parts, state: &G) -> Result<Self, Self::Rejection> {
        if let Ok(GameCode { code }) = parts.extract::<GameCode>().await {
            return Ok(Game(state.get(GameBy::Code(code)).await));
        } else if let Ok(GameID { game_id }) = parts.extract::<GameID>().await {
            return Ok(Game(state.get(GameBy::ID(game_id)).await));
        } else {
            Err(anyhow!("failed to extract game_id or code from path"))?
        }
    }
}
