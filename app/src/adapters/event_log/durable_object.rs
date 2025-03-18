use gloo_utils::format::JsValueSerdeExt;
use shared::time::{Duration, SystemTime};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsValue;

use anyhow::Result;
use im::Vector;
use shared::{models::events::Event, time};
use tracing::instrument;
use worker::{ListOptions, Storage};

use crate::ports::event_log::EventLog;

struct Inner {
    storage: Storage,
    events: Vector<Event>,
    hydrated: bool,
}

#[derive(Clone)]
pub struct DurableObjectKeyValue {
    inner: Rc<RefCell<Inner>>,
}

// Safety: wasm32 is single threaded but axum doesn't know that
// DurableObjectKeyValue isn't Send + Sync because Storage contains a JsObject which is a *mut u32
#[cfg(target_arch = "wasm32")]
unsafe impl Send for DurableObjectKeyValue {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for DurableObjectKeyValue {}

impl DurableObjectKeyValue {
    pub fn new(storage: Storage) -> DurableObjectKeyValue {
        DurableObjectKeyValue {
            inner: Rc::new(RefCell::new(Inner {
                storage,
                events: Vector::new(),
                hydrated: false,
            })),
        }
    }

    #[instrument(skip_all, err)]
    async fn hydrate(&self) -> worker::Result<()> {
        let mut this = (*self.inner).borrow_mut();

        if this.hydrated {
            return Ok(());
        }

        let events = this
            .storage
            .list_with_options(ListOptions::new().prefix("EVENT#"))
            .await?;

        events
            .values()
            .into_iter()
            .try_for_each::<_, worker::Result<_>>(&mut |value: Result<JsValue, JsValue>| {
                let value = value?;

                let event = JsValueSerdeExt::into_serde(&value).inspect_err(|err| {
                    tracing::error!(
                        ?err,
                        ?value,
                        "failed to parse value from log during hydration"
                    );
                })?;

                this.events.push_back(event);

                Ok(())
            })?;

        this.hydrated = true;

        Ok(())
    }

    pub async fn write_alarm(&self, alarm: Duration) -> Result<()> {
        let wakeup = time::SystemTime::now() + alarm;

        (*self.inner)
            .borrow_mut()
            .storage
            .put(
                "ALARM",
                &wakeup.duration_since(time::UNIX_EPOCH)?.as_secs_f64(),
            )
            .await?;

        Ok(())
    }

    pub async fn read_alarm(&self) -> Option<SystemTime> {
        let time = (*self.inner)
            .borrow_mut()
            .storage
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

        let mut this = (*self.inner).borrow_mut();

        let key = format!("EVENT#{:0>5}", this.events.len());

        this.storage.put(&key, &event).await?;
        this.events.push_back(event);

        Ok(())
    }

    #[instrument(skip_all)]
    async fn iter(&self) -> Result<impl Iterator<Item = Event>> {
        self.hydrate().await?;

        Ok((*self.inner).borrow().events.clone().into_iter())
    }

    #[instrument(skip_all)]
    async fn vector(&self) -> Result<Vector<Event>> {
        self.hydrate().await?;

        Ok((*self.inner).borrow().events.clone())
    }
}
