use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

use anyhow::Result;
use axum::extract::ws::WebSocket;
use shared::models::{events::Event, game_id::GameID};

use crate::{
    adapters::event_log::{file::FileEventLog, in_memory::InMemoryKV},
    ports::{event_log::EventLog, game_state::GameState},
};

struct Game {
    events: FileEventLog,
    sockets: Vec<WebSocket>,
}

impl Game {
    fn from_game_id(game_id: GameID) -> Self {
        Self {
            events: FileEventLog::from_game_id(game_id),
            sockets: vec![],
        }
    }
}

#[derive(Clone, Default)]
pub struct FileGameState {
    inner: Arc<Mutex<HashMap<GameID, Game>>>,
}

impl GameState for FileGameState {
    async fn events(&self, game_id: GameID) -> Result<im::Vector<Event>> {
        self.inner
            .lock()
            .await
            .entry(game_id)
            .or_insert_with(|| Game::from_game_id(game_id))
            .events
            .vector()
            .await
    }

    async fn push_event(&self, game_id: GameID, event: Event) -> Result<()> {
        let mut lock_guard = self.inner.lock().await;
        let game = lock_guard
            .entry(game_id)
            .or_insert_with(|| Game::from_game_id(game_id));

        game.events.push(event.clone()).await?;

        for mut socket in std::mem::take(&mut game.sockets) {
            let message = serde_json::to_string(&event)?;

            if let Err(err) = socket.send(message.into()).await {
                tracing::warn!(
                    ?err,
                    "error forwarding event to client sockets, removing socket"
                );

                continue;
            }

            game.sockets.push(socket);
        }

        Ok(())
    }

    async fn accept_web_socket(&self, game_id: GameID, ws: WebSocket) -> Result<()> {
        self.inner
            .lock()
            .await
            .entry(game_id)
            .or_insert_with(|| Game::from_game_id(game_id))
            .sockets
            .push(ws);

        Ok(())
    }
}
