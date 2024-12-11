use std::result::Result::Ok;

use anyhow::Result;
use shared::models::{events::Event, game_code::GameCode};
use tokio::io::AsyncWriteExt;
use tracing::instrument;

use crate::ports::event_log::EventLog;

#[derive(Debug, Clone)]
pub struct FileEventLog {
    path: String,
}

impl FileEventLog {
    pub fn from_game_id(game_id: GameCode) -> Self {
        Self {
            path: format!(".game_state/{game_id}.json"),
        }
    }
}

impl Default for FileEventLog {
    fn default() -> Self {
        Self {
            path: ".game_state/default.json".to_string(),
        }
    }
}

impl EventLog for FileEventLog {
    #[instrument(err)]
    async fn push(&self, event: Event) -> Result<()> {
        let mut event_log = self.vector().await?;
        event_log.push_back(event);

        let data = serde_json::to_string(&Vec::from_iter(&event_log))?;

        let mut file = tokio::fs::File::create(&self.path).await?;
        file.write_all(data.as_bytes()).await?;

        Ok(())
    }

    #[instrument(err)]
    async fn iter(&self) -> Result<impl Iterator<Item = Event>> {
        Ok(self.vector().await?.into_iter())
    }

    #[instrument(err)]
    async fn vector(&self) -> Result<im::Vector<Event>> {
        let data = tokio::fs::read_to_string(&self.path)
            .await
            .unwrap_or_else(|_| "[]".to_string());

        let event_log = serde_json::from_str::<Vec<Event>>(&data)?;

        Ok(event_log.into())
    }
}
