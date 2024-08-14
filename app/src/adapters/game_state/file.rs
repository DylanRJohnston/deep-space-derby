use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle};

use anyhow::Result;
use axum::extract::ws::WebSocket;
use shared::models::{events::Event, game_id::GameID, processors::run_processors};
use tracing::instrument;

use crate::{
    adapters::event_log::file::FileEventLog,
    ports::{event_log::EventLog, game_state::GameState},
};

struct Game {
    events: FileEventLog,
    sockets: Vec<WebSocket>,
    alarm: Option<JoinHandle<()>>,
}

impl Game {
    async fn push_event(&mut self, event: Event) -> Result<()> {
        self.events.push(event.clone()).await?;

        for mut socket in std::mem::take(&mut self.sockets) {
            let message = serde_json::to_string(&event)?;

            if let Err(err) = socket.send(message.into()).await {
                tracing::warn!(
                    ?err,
                    "error forwarding event to client sockets, removing socket"
                );

                continue;
            }

            self.sockets.push(socket);
        }

        Ok(())
    }
}

impl Game {
    fn from_game_id(game_id: GameID) -> Self {
        Self {
            events: FileEventLog::from_game_id(game_id),
            sockets: vec![],
            alarm: None,
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

        game.push_event(event).await?;

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

    #[instrument(skip(self))]
    fn set_alarm(
        &self,
        game_id: GameID,
        duration: Duration,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let mut lock_guard = self.inner.lock().await;

            let game = lock_guard
                .entry(game_id)
                .or_insert_with(|| Game::from_game_id(game_id));

            if let Some(handle) = game.alarm.take() {
                handle.abort();
            }

            let this = Arc::downgrade(&self.inner);

            game.alarm = Some(tokio::spawn(async move {
                let result: anyhow::Result<()> = try {
                    tokio::time::sleep(duration).await;

                    let Some(this) = this.upgrade() else {
                        tracing::warn!("alarm trigger after game state was dropped");
                        return;
                    };

                    let mut lock_guard = this.lock().await;

                    let game = lock_guard
                        .entry(game_id)
                        .or_insert_with(|| Game::from_game_id(game_id));

                    let (new_events, alarm) = run_processors(&game.events.vector().await?)?;

                    for event in new_events {
                        game.push_event(event).await?;
                    }

                    game.alarm = None;
                    drop(lock_guard);

                    if let Some(alarm) = alarm {
                        FileGameState { inner: this }
                            .set_alarm(game_id, alarm.0)
                            .await?;
                    }
                };

                if let Err(err) = result {
                    tracing::error!(?err, "error encountered waking up from alarm");
                }
            }));

            Ok(())
        })
    }
}
