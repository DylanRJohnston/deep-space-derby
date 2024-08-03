use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::Result;
use axum::extract::ws::WebSocket;
use shared::models::events::Event;

use crate::{
    adapters::event_log::in_memory::InMemoryKV,
    ports::{event_log::EventLog, game_state::GameState},
};

#[derive(Clone, Default)]
pub struct InMemoryGameState {
    events: InMemoryKV,
    sockets: Arc<Mutex<Vec<WebSocket>>>,
}

impl GameState for InMemoryGameState {
    async fn events(&self) -> Result<im::Vector<Event>> {
        self.events.vector().await
    }

    async fn push_event(&self, event: Event) -> Result<()> {
        self.events.push(event.clone()).await?;

        for socket in self.sockets.lock().await.iter_mut() {
            let message = serde_json::to_string(&event)?;

            if let Err(err) = socket.send(message.into()).await {
                tracing::error!(?err, "error forwarding event to client sockets")
            }
        }

        Ok(())
    }

    async fn accept_web_socket(&self, ws: WebSocket) -> Result<()> {
        self.sockets.lock().await.push(ws);

        Ok(())
    }
}
