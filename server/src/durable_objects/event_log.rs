use im::Vector;
use shared::models::events::Event;
use tracing::instrument;
use worker::{ListOptions, Storage};

pub struct EventLog {
    storage: Storage,
    events: Vector<Event>,
    hydrated: bool,
}

impl EventLog {
    pub fn new(storage: Storage) -> EventLog {
        EventLog {
            storage,
            events: Vector::new(),
            hydrated: false,
        }
    }

    #[instrument(skip_all, err)]
    pub async fn hydrate(&mut self) -> worker::Result<()> {
        if self.hydrated {
            return Ok(());
        }

        let events = self
            .storage
            .list_with_options(ListOptions::new().prefix("EVENT#"))
            .await?;

        events
            .values()
            .into_iter()
            .try_for_each::<_, worker::Result<_>>(&mut |value| {
                let event = serde_wasm_bindgen::from_value::<Event>(value?)?;

                self.events.push_back(event);

                Ok(())
            })?;

        self.hydrated = true;

        Ok(())
    }

    #[instrument(skip_all, err)]
    pub async fn push(&mut self, event: Event) -> worker::Result<()> {
        self.hydrate().await?;

        let key = format!("EVENT#{:0>5}", self.events.len());

        self.storage.put(&key, &event).await?;
        self.events.push_back(event);

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn iter(&mut self) -> worker::Result<impl Iterator<Item = &Event> + '_> {
        self.hydrate().await?;

        Ok(self.events.iter())
    }

    #[instrument(skip_all)]
    pub async fn vector(&mut self) -> worker::Result<&Vector<Event>> {
        self.hydrate().await?;

        Ok(&self.events)
    }
}
