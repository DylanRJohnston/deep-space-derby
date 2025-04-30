use std::future::Future;

use anyhow::Result;
use im::Vector;
use shared::models::events::Event;

pub trait EventLog {
    fn push(&self, event: Event) -> impl Future<Output = Result<()>>;
    fn iter(&self) -> impl Future<Output = Result<impl Iterator<Item = Event>>>;
    fn vector(&self) -> impl Future<Output = Result<Vector<Event>>>;
}
