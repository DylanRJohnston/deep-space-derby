use std::fmt::Debug;
use std::{
    future::Future,
    ops::Deref,
    sync::{Arc, RwLock},
};

use bevy::ecs::component::Component;
use bevy::tasks::IoTaskPool;

#[derive(Debug, Component)]
pub struct AsyncTask<T>(Arc<RwLock<Option<T>>>);

impl<T> Clone for AsyncTask<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub trait TaskTrait<T>: Future<Output = T> + Send + 'static {}

#[cfg(not(target_arch = "wasm32"))]
impl<T, F: Future<Output = T> + Send + 'static> TaskTrait<T> for F {}

#[cfg(target_arch = "wasm32")]
pub trait TaskTrait<T>: Future<Output = T> + 'static {}

#[cfg(target_arch = "wasm32")]
impl<T, F: Future<Output = T> + 'static> TaskTrait<T> for F {}

impl<T: Send + Sync + 'static + Debug> AsyncTask<T> {
    fn new() -> Self {
        AsyncTask(Arc::new(RwLock::new(None::<T>)))
    }

    async fn register(&mut self, t: impl Future<Output = T>) {
        let result = t.await;

        let mut lock: std::sync::RwLockWriteGuard<'_, Option<T>> = self.0.write().unwrap();

        *lock = Some(result);
    }

    pub fn on_completion(&self, f: impl FnOnce(&T)) {
        let lock = self.0.read().unwrap();

        if let Some(t) = lock.deref() {
            f(t);
        };
    }

    pub fn spawn(t: impl TaskTrait<T>) -> AsyncTask<T> {
        let task: AsyncTask<T> = AsyncTask::new();

        let mut task_clone = task.clone();
        IoTaskPool::get()
            .spawn(async move {
                task_clone.register(t).await;
            })
            .detach();

        task
    }
}
