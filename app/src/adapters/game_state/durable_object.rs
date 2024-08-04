use std::rc::Rc;

use shared::models::events::Event;
use shared::models::game_id::GameID;
use tower::Service;
use tracing::instrument;
use worker::send::SendFuture;
use worker::{durable_object, Env, State, WebSocketPair};

use crate::adapters::event_log::durable_object::DurableObjectKeyValue;
use crate::ports::event_log::EventLog;
use crate::ports::game_state::GameState;
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
}

// DurableObject Game State Can Ignore the GameID parameter as there is one per game
impl GameState for Game {
    async fn events(&self, _: GameID) -> anyhow::Result<im::Vector<Event>> {
        SendFuture::new(async { Ok(self.events.vector().await?) }).await
    }

    async fn push_event(&self, _: GameID, event: Event) -> anyhow::Result<()> {
        SendFuture::new(async {
            self.events.push(event.clone()).await;

            for ws in self.state.get_websockets() {
                ws.send(&event)?;
            }

            Ok(())
        })
        .await
    }

    fn accept_web_socket(&self, _: GameID) -> anyhow::Result<WebSocketPair> {
        let pair = WebSocketPair::new()?;
        self.state.accept_web_socket(&pair.server);

        Ok(pair)
    }
}
