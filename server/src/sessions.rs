use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub username: String,
    pub reloaded_count: i32,
}

#[derive(Debug)]
pub struct Session {
    pub metadata: Metadata,
    pub socket: WebSocket,
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.socket == other.socket
    }
}

#[derive(Debug)]
pub struct Sessions(Vec<Session>);

impl Sessions {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, session: Session) {
        self.0.push(session)
    }

    pub fn get(&self, ws: &WebSocket) -> Option<&Session> {
        self.0.iter().find(|it| &it.socket == ws)
    }

    pub fn remove(&mut self, ws: &WebSocket) -> Option<Session> {
        if let Some(position) = self.0.iter().position(|it| &it.socket == ws) {
            return Some(self.0.remove(position));
        }

        None
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &Session> {
        self.0.iter()
    }
}

impl Default for Sessions {
    fn default() -> Self {
        Self::new()
    }
}
