use std::ops::{Deref, DerefMut};

use axum::routing::get;
use shared::models::commands;
use shared::models::events::Event;
use tower::Service;
use tracing::instrument;
use worker::{durable_object, Env, State, WebSocket};

use crate::durable_objects::event_log::EventLog;
use crate::handlers::on_connect::on_connect;
use crate::handlers::register_command::CommandHandler;

#[durable_object]
pub struct Game {
    state: State,
    events: EventLog,
    env: Env,
    // sessions: Sessions,
}

#[durable_object]
impl DurableObject for Game {
    pub fn new(state: State, env: Env) -> Self {
        let events = EventLog::new(state.storage());

        Self {
            state,
            events,
            env,
            // sessions,
        }
    }

    #[instrument(name = "Game::fetch", skip_all)]
    pub async fn fetch(&mut self, req: worker::Request) -> worker::Result<worker::Response> {
        tracing::info!("request made it to durable object");

        axum::Router::new()
            .route("/api/object/game/by_code/:code/connect", get(on_connect))
            .register_command::<commands::CreateGame>()
            .register_command::<commands::JoinGame>()
            .register_command::<commands::ChangeProfile>()
            .register_command::<commands::ReadyPlayer>()
            .register_command::<commands::PlaceBets>()
            .with_state(GameWrapper::new(self))
            .call(req.try_into()?)
            .await?
            .try_into()
    }

    pub async fn websocket_close(
        &mut self,
        ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> worker::Result<()> {
        // self.sessions.remove(&ws);

        Ok(())
    }

    pub async fn websocket_error(
        &mut self,
        ws: WebSocket,
        _error: worker::Error,
    ) -> worker::Result<()> {
        // self.sessions.remove(&ws);

        Ok(())
    }
}

impl Game {
    #[instrument(skip_all)]
    pub async fn add_event(&mut self, event: Event) -> worker::Result<()> {
        self.events.push(event.clone()).await?;

        for ws in &self.state.get_websockets() {
            ws.send(&event)?;
        }

        Ok(())

        // self.sessions.broadcast(&event)
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn events(&mut self) -> &mut EventLog {
        &mut self.events
    }
}

// Workers are single threaded, but axum has annoying Send + Send + 'static bounds on State
pub struct GameWrapper(*mut Game);

unsafe impl Sync for GameWrapper {}
unsafe impl Send for GameWrapper {}

impl GameWrapper {
    fn new(game: &mut Game) -> Self {
        Self(game as *mut Game)
    }
}

impl Clone for GameWrapper {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Deref for GameWrapper {
    type Target = Game;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for GameWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}
