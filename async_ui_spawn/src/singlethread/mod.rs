mod spawner;

use std::{future::Future, pin::Pin};

use smol::{LocalExecutor, Task};

pub trait Send {}
impl<T> Send for T {}

thread_local! {
    static EXECUTOR: LocalExecutor<'static> = LocalExecutor::new();
}
fn spawn<F>(future: F) -> Task<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    EXECUTOR.with(|exe| exe.spawn(future))
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub use spawner::SpawnedFuture;
