use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle};

use anyhow::Result;
use axum::extract::ws::WebSocket;
use shared::models::{
    events::{Event, EventStream},
    game_code::GameCode,
};

use crate::{
    adapters::event_log::file::FileEventLog,
    ports::{
        event_log::EventLog,
        game_service::GameBy,
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
            let message = serde_json::to_string(&EventStream::Event(event.clone()))?;

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
    fn from_game_code(game_code: GameCode) -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerGame {
                events: FileEventLog::from_game_id(game_code),
                sockets: vec![],
                alarm: None,
            })),
        }
    }
}

#[derive(Clone, Default)]
pub struct FileGameDirectory {
    inner: Arc<Mutex<HashMap<GameCode, Game>>>,
}

impl GameDirectory for FileGameDirectory {
    type WebSocket = WebSocket;
    type GameState = Game;

    // The lock on the hashmap is only held while we figure out if the game exists or not
    async fn get(&self, game_id: GameBy) -> Self::GameState {
        let GameBy::Code(game_id) = game_id else {
            panic!("FileGameDirectory::get() called with GameBy::Code");
        };

        self.inner
            .lock()
            .await
            .entry(game_id)
            .or_insert_with(|| Game::from_game_code(game_id))
            .clone()
    }
}

// Error

// A separate module is required to solve problems with the compiler not knowing if the hidden type
// of the opaque type satisfies the auto trait bounds.
mod set_alarm {
    use anyhow::Result;
    use shared::models::processors::run_processors;
    use std::{future::Future, sync::Arc, time::Duration};

    use crate::ports::{event_log::EventLog, game_state::GameState};

    use super::Game;

    pub fn set_alarm(
        this: &Game,
        duration: Duration,
    ) -> impl Future<Output = Result<()>> + Send + '_ {
        async move {
            let mut game = this.inner.lock().await;

            if let Some(handle) = game.alarm.take() {
                handle.abort();
            }

            let this = Arc::downgrade(&this.inner);

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
        }
    }
}

impl GameState for Game {
    type WebSocket = WebSocket;

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

    async fn set_alarm(&self, duration: Duration) -> Result<()> {
        set_alarm::set_alarm(&self, duration).await
    }
}
