mod spawner;

use std::{future::Future, pin::Pin};

use async_executor::Executor;
pub type Task = async_executor::Task<()>;

static EXECUTOR: Executor = Executor::new();
type SpawnJob = SpawnWrappedFuture<dyn Future<Output = ()> + Send + 'static>;
fn spawn(future: SpawnJob) -> Task {
    EXECUTOR.spawn(future)
}

use std::marker::Send;
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub use spawner::*;

use crate::shared::SpawnWrappedFuture;
