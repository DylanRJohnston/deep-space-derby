use shared::time::{Duration, SystemTime};
use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use im::Vector;
use shared::{models::events::Event, time};
use tracing::instrument;
use worker::{ListOptions, Storage};

use crate::ports::event_log::EventLog;

struct Inner {
    storage: RefCell<Storage>,
    events: RefCell<Vector<Event>>,
    hydrated: RefCell<bool>,
}

#[derive(Clone)]
pub struct DurableObjectKeyValue {
    inner: Rc<Inner>,
}

// Safety: DOKV is only constructable in WASM
unsafe impl Send for DurableObjectKeyValue {}
unsafe impl Sync for DurableObjectKeyValue {}

impl DurableObjectKeyValue {
    pub fn new(storage: Storage) -> DurableObjectKeyValue {
        DurableObjectKeyValue {
            inner: Rc::new(Inner {
                storage: RefCell::new(storage),
                events: RefCell::new(Vector::new()),
                hydrated: RefCell::new(false),
            }),
        }
    }

    #[instrument(skip_all, err)]
    async fn hydrate(&self) -> worker::Result<()> {
        if *self.inner.hydrated.borrow() {
            return Ok(());
        }

        let events = self
            .inner
            .storage
            .borrow_mut()
            .list_with_options(ListOptions::new().prefix("EVENT#"))
            .await?;

        events
            .values()
            .into_iter()
            .try_for_each::<_, worker::Result<_>>(&mut |value| {
                let event = serde_wasm_bindgen::from_value::<Event>(value?)?;

                self.inner.events.borrow_mut().push_back(event);

                Ok(())
            })?;

        *self.inner.hydrated.borrow_mut() = true;

        Ok(())
    }

    pub async fn write_alarm(&self, alarm: Duration) -> Result<()> {
        let wakeup = time::SystemTime::now() + alarm;

        self.inner
            .storage
            .borrow_mut()
            .put(
                "ALARM",
                &wakeup.duration_since(time::UNIX_EPOCH)?.as_secs_f64(),
            )
            .await?;

        Ok(())
    }

    pub async fn read_alarm(&self) -> Option<SystemTime> {
        let time = self
            .inner
            .storage
            .borrow_mut()
            .get::<f64>("ALARM")
            .await
            .ok()?;

        Some(SystemTime::UNIX_EPOCH + Duration::from_secs_f64(time))
    }
}

impl EventLog for DurableObjectKeyValue {
    #[instrument(skip_all, err)]
    async fn push(&self, event: Event) -> Result<()> {
        self.hydrate().await?;

        let key = format!("EVENT#{:0>5}", self.inner.events.borrow().len());

        self.inner.storage.borrow_mut().put(&key, &event).await?;
        self.inner.events.borrow_mut().push_back(event);

        Ok(())
    }

    #[instrument(skip_all)]
    async fn iter(&self) -> Result<impl Iterator<Item = Event>> {
        self.hydrate().await?;

        Ok(self.inner.events.borrow().clone().into_iter())
    }

    #[instrument(skip_all)]
    async fn vector(&self) -> Result<Vector<Event>> {
        self.hydrate().await?;

        Ok(self.inner.events.borrow().clone())
    }
}
