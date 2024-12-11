use std::rc::Rc;

use anyhow::Result;
use im::Vector;
use shared::models::events::{Event, EventStream};
use shared::models::processors::{run_processors, Alarm};
use shared::time::SystemTime;
use tower::Service;
use tracing::instrument;
use worker::{durable_object, Env, State, WebSocket};
use worker_macros::send;

use crate::adapters::event_log::durable_object::DurableObjectKeyValue;
use crate::ports::event_log::EventLog;
use crate::ports::game_service::GameBy;
use crate::ports::game_state::{GameDirectory, GameState};
use crate::router::into_game_router;

#[derive(Clone)]
#[durable_object]
pub struct Game {
    state: Rc<State>,
    events: DurableObjectKeyValue,
    // sessions: Sessions,
}

unsafe impl Send for Game {}
unsafe impl Sync for Game {}

#[durable_object]
impl DurableObject for Game {
    pub fn new(state: State, _env: Env) -> Self {
        let events = DurableObjectKeyValue::new(state.storage());
        Self {
            events,
            state: Rc::new(state), // sessions,
        }
    }

    #[instrument(name = "Game::fetch", skip_all)]
    pub async fn fetch(&mut self, req: worker::Request) -> worker::Result<worker::Response> {
        Ok(into_game_router(self.clone())
            .call(req.try_into()?)
            .await?
            .try_into()?)
    }

    pub async fn alarm(&mut self) -> worker::Result<worker::Response> {
        if let Some(alarm) = self.events.read_alarm().await {
            let now = SystemTime::now();
            if now < alarm {
                let delta = alarm.duration_since(now).map_err(|err| err.to_string())?;

                tracing::warn!(?delta, "woke up too early, going back to sleep");

                self.set_alarm(delta).await.map_err(|err| err.to_string())?;

                return worker::Response::empty();
            }
        };

        let events = self.events.vector().await.map_err(|err| err.to_string())?;

        let (events, alarm) = run_processors(&events).map_err(|err| err.to_string())?;

        for event in events {
            self.push_event(event)
                .await
                .map_err(|err| err.to_string())?;
        }

        if let Some(Alarm(duration)) = alarm {
            self.set_alarm(duration)
                .await
                .map_err(|err| err.to_string())?;
        }

        worker::Response::empty()
    }
}

// DurableObject Game State Can Ignore the GameID parameter as there is one per game
impl GameState for Game {
    type WebSocket = WebSocket;

    #[send]
    async fn events(&self) -> Result<Vector<Event>> {
        Ok(self.events.vector().await?)
    }

    #[send]
    async fn push_event(&self, event: Event) -> Result<()> {
        self.events.push(event.clone()).await?;

        for ws in self.state.get_websockets() {
            ws.send(&EventStream::Event(event.clone()))?;
        }

        Ok(())
    }

    async fn accept_web_socket(&self, ws: WebSocket) -> Result<()> {
        self.state.accept_web_socket(&ws);

        Ok(())
    }

    #[send]
    async fn set_alarm(&self, duration: std::time::Duration) -> Result<()> {
        self.events.write_alarm(duration.clone()).await?;

        let storage = self.state.storage();

        if let Some(time) = storage.get_alarm().await? {
            tracing::warn!(?time, "overriding timer");
        };

        storage.set_alarm(duration).await?;

        Ok(())
    }
}

// DurableObjects are already singletons, so there's nothing to do here
impl GameDirectory for Game {
    type WebSocket = WebSocket;
    type GameState = Game;

    async fn get(&self, _: GameBy) -> Self::GameState {
        self.clone()
    }
}
