use std::{future::Future, pin::Pin, time::Duration};

use anyhow::Result;
use im::Vector;
use shared::models::{events::Event, game_id::GameID};

pub trait GameState: Clone + Send + Sync + 'static {
    fn events(&self, game_id: GameID) -> impl Future<Output = Result<Vector<Event>>> + Send;
    fn push_event(&self, game_id: GameID, event: Event) -> impl Future<Output = Result<()>> + Send;
    fn set_alarm(
        &self,
        game_id: GameID,
        duration: Duration,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    #[cfg(target_arch = "wasm32")]
    fn accept_web_socket(&self, game_id: GameID) -> anyhow::Result<worker::WebSocketPair>;

    #[cfg(not(target_arch = "wasm32"))]
    fn accept_web_socket(
        &self,
        game_id: GameID,
        ws: axum::extract::ws::WebSocket,
    ) -> impl Future<Output = Result<()>> + Send;
}
