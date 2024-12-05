use std::{future::Future, time::Duration};

use anyhow::Result;
use im::Vector;
use shared::models::{events::Event, game_id::GameID};

pub trait GameState: Clone + Send + 'static {
    type WebSocket;

    fn events(&self) -> impl Future<Output = Result<Vector<Event>>> + Send;
    fn push_event(&self, event: Event) -> impl Future<Output = Result<()>> + Send;
    fn set_alarm(&self, duration: Duration) -> impl Future<Output = Result<()>> + Send;

    fn accept_web_socket(&self, ws: Self::WebSocket) -> impl Future<Output = Result<()>> + Send;
}

pub trait GameDirectory: Clone + Send + Sync + 'static {
    type WebSocket;
    type GameState: GameState<WebSocket = Self::WebSocket>;

    fn get(&self, game_id: GameID) -> impl Future<Output = Self::GameState> + Send;
}
