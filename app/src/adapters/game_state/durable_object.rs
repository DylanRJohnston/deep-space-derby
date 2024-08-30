use std::rc::Rc;

use shared::models::events::Event;
use shared::models::game_id::GameID;
use shared::models::processors::{run_processors, Alarm};
use tower::Service;
use tracing::instrument;
use worker::send::SendFuture;
use worker::{durable_object, Env, State, Storage, WebSocketPair};

use crate::adapters::event_log::durable_object::DurableObjectKeyValue;
use crate::ports::event_log::EventLog;
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
    pub fn new(state: State, env: Env) -> Self {
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

        worker::Response::ok("")
    }
}

// DurableObject Game State Can Ignore the GameID parameter as there is one per game
impl GameState for Game {
    async fn events(&self) -> anyhow::Result<im::Vector<Event>> {
        SendFuture::new(async { Ok(self.events.vector().await?) }).await
    }

    async fn push_event(&self, event: Event) -> anyhow::Result<()> {
        SendFuture::new(async {
            self.events.push(event.clone()).await;

            for ws in self.state.get_websockets() {
                ws.send(&event)?;
            }

            Ok(())
        })
        .await
    }

    fn accept_web_socket(&self) -> anyhow::Result<WebSocketPair> {
        let pair = WebSocketPair::new()?;
        self.state.accept_web_socket(&pair.server);

        Ok(pair)
    }

    fn set_alarm(
        &self,
        duration: std::time::Duration,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move {
            SendFuture::new(async {
                let storage = self.state.storage();

                if let Some(time) = storage.get_alarm().await? {
                    tracing::warn!(?time, "overriding timer");
                };

                storage.set_alarm(duration).await?;

                Ok(())
            })
            .await
        })
    }
}

impl GameDirectory for Game {
    type GameState = Game;

    async fn get(&self, _: GameID) -> Self::GameState {
        self.clone()
    }
}
