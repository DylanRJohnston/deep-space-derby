use std::{collections::HashMap, pin::Pin, sync::Arc};
use tokio::sync::{Mutex, RwLock};

use anyhow::Result;
use axum::extract::ws::WebSocket;
use shared::models::{events::Event, game_id::GameID};

use crate::{
    adapters::event_log::in_memory::InMemoryKV,
    ports::{event_log::EventLog, game_state::GameState},
};

#[derive(Clone, Default)]
struct Game {
    events: InMemoryKV,
    sockets: Arc<Mutex<Vec<WebSocket>>>,
}

#[derive(Clone, Default)]
pub struct InMemoryGameState {
    inner: Arc<RwLock<HashMap<GameID, Game>>>,
}

impl GameState for InMemoryGameState {
    async fn events(&self, game_id: GameID) -> Result<im::Vector<Event>> {
        self.inner
            .read()
            .await
            .get(&game_id)
            .cloned()
            .unwrap_or_default()
            .events
            .vector()
            .await
    }

    async fn push_event(&self, game_id: GameID, event: Event) -> Result<()> {
        let mut lock_guard = self.inner.write().await;
        let game = lock_guard.entry(game_id).or_default();

        game.events.push(event.clone()).await?;

        for socket in game.sockets.lock().await.iter_mut() {
            let message = serde_json::to_string(&event)?;

            if let Err(err) = socket.send(message.into()).await {
                tracing::error!(?err, "error forwarding event to client sockets")
            }
        }

        Ok(())
    }

    async fn accept_web_socket(&self, game_id: GameID, ws: WebSocket) -> Result<()> {
        let mut lock_guard = self.inner.write().await;
        let game = lock_guard.entry(game_id).or_default();

        game.sockets.lock().await.push(ws);

        Ok(())
    }

    fn set_alarm(
        &self,
        _: GameID,
        _: std::time::Duration,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        unimplemented!()
    }
}
