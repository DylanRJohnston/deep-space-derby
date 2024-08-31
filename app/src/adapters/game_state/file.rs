use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle};

use anyhow::Result;
use axum::extract::ws::WebSocket;
use shared::models::{events::Event, game_id::GameID, processors::run_processors};
use tracing::instrument;

use crate::{
    adapters::event_log::file::FileEventLog,
    ports::{
        event_log::EventLog,
        game_state::{GameDirectory, GameState},
    },
};

struct InnerGame {
    events: FileEventLog,
    sockets: Vec<WebSocket>,
    alarm: Option<JoinHandle<()>>,
}

#[derive(Clone)]
pub struct Game {
    inner: Arc<Mutex<InnerGame>>,
}

impl InnerGame {
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
            inner: Arc::new(Mutex::new(InnerGame {
                events: FileEventLog::from_game_id(game_id),
                sockets: vec![],
                alarm: None,
            })),
        }
    }
}

#[derive(Clone, Default)]
pub struct FileGameDirectory {
    inner: Arc<Mutex<HashMap<GameID, Game>>>,
}

impl GameDirectory for FileGameDirectory {
    type GameState = Game;

    // The lock on the hashmap is only held while we figure out if the game exists or not
    async fn get(&self, game_id: GameID) -> Self::GameState {
        self.inner
            .lock()
            .await
            .entry(game_id)
            .or_insert_with(|| Game::from_game_id(game_id))
            .clone()
    }
}

impl GameState for Game {
    async fn events(&self) -> Result<im::Vector<Event>> {
        self.inner.lock().await.events.vector().await
    }

    async fn push_event(&self, event: Event) -> Result<()> {
        let mut lock_guard = self.inner.lock().await;

        lock_guard.push_event(event).await?;

        Ok(())
    }

    async fn accept_web_socket(&self, ws: WebSocket) -> Result<()> {
        self.inner.lock().await.sockets.push(ws);

        Ok(())
    }

    #[instrument(skip(self))]
    fn set_alarm(
        &self,
        duration: Duration,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let mut game = self.inner.lock().await;

            if let Some(handle) = game.alarm.take() {
                handle.abort();
            }

            let this = Arc::downgrade(&self.inner);

            tracing::info!(?duration, "setting alarm");
            game.alarm = Some(tokio::spawn(async move {
                let result: anyhow::Result<()> = try {
                    tokio::time::sleep(duration).await;

                    tracing::info!("waking up from alarm");

                    let Some(this) = this.upgrade() else {
                        tracing::warn!("alarm trigger after game state was dropped");
                        return;
                    };

                    let mut game = this.lock().await;

                    let (new_events, alarm) = run_processors(&game.events.vector().await?)?;

                    tracing::info!(?new_events, ?alarm, "alarm events");

                    for event in new_events {
                        game.push_event(event).await?;
                    }

                    game.alarm = None;
                    drop(game);

                    if let Some(alarm) = alarm {
                        Game { inner: this }.set_alarm(alarm.0).await?;
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
