use serde::{Deserialize, Serialize};
use uuid::Uuid;
use worker::*;

use crate::models::events::Event;

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub session_id: Uuid,
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

    pub fn remove(&mut self, ws: &WebSocket) -> Option<Session> {
        if let Some(position) = self.0.iter().position(|it| &it.socket == ws) {
            return Some(self.0.remove(position));
        }

        None
    }

    pub fn iter(&self) -> impl Iterator<Item = &Session> {
        self.0.iter()
    }

    pub fn broadcast(&self, data: &Event) -> Result<()> {
        for session in self.iter() {
            session.socket.send(data)?;
        }

        Ok(())
    }
}

impl Default for Sessions {
    fn default() -> Self {
        Self::new()
    }
}
