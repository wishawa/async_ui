mod spawner;

use std::{future::Future, pin::Pin};

use async_executor::LocalExecutor;

thread_local! {
    static EXECUTOR: LocalExecutor<'static> = LocalExecutor::new();
}
type SpawnJob = SpawnWrappedFuture<dyn Future<Output = ()> + 'static>;
pub type Task = async_executor::Task<()>;
fn spawn(future: SpawnJob) -> Task {
    EXECUTOR.with(|exe| exe.spawn(future))
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub use spawner::*;

use crate::shared::SpawnWrappedFuture;
