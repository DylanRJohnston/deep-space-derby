use std::future::Future;

use anyhow::Result;
use im::Vector;
use shared::models::events::Event;

pub trait GameState: Clone + Send + Sync + 'static {
    fn events(&self) -> impl Future<Output = Result<Vector<Event>>> + Send;
    fn push_event(&self, event: Event) -> impl Future<Output = Result<()>> + Send;

    #[cfg(target_arch = "wasm32")]
    fn accept_web_socket(&self) -> anyhow::Result<worker::WebSocketPair>;

    #[cfg(not(target_arch = "wasm32"))]
    fn accept_web_socket(
        &self,
        ws: axum::extract::ws::WebSocket,
    ) -> impl Future<Output = Result<()>> + Send;
}
