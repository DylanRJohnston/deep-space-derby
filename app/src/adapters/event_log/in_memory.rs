use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Ok, Result};
use im::Vector;
use shared::models::events::Event;

use crate::ports::event_log::EventLog;

#[derive(Clone, Default)]
pub struct InMemoryKV(Arc<RwLock<Vector<Event>>>);

impl EventLog for InMemoryKV {
    async fn push(&self, event: Event) -> Result<()> {
        self.0
            .write()
            .map_err(|_| anyhow!("lock poisoned"))?
            .push_back(event);

        Ok(())
    }

    async fn iter(&self) -> Result<impl Iterator<Item = Event>> {
        Ok(self
            .0
            .read()
            .map_err(|_| anyhow!("lock poisoned"))?
            .clone()
            .into_iter())
    }

    async fn vector(&self) -> Result<im::Vector<Event>> {
        Ok(self.0.read().map_err(|_| anyhow!("lock poisoned"))?.clone())
    }
}
