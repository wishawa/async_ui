mod spawner;

use std::{future::Future, pin::Pin};

use smol::{Executor, Task};

static EXECUTOR: Executor = Executor::new();
fn spawn<F>(future: F) -> Task<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    EXECUTOR.spawn(future)
}

use std::marker::Send;
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub use spawner::SpawnedFuture;
