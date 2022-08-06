use async_executor::Task;
use scoped_async_spawn::ExecutorSpawn;

use crate::executor::spawn_local;

#[derive(Clone)]
pub(super) struct Spawner;
impl ExecutorSpawn for Spawner {
    type Task = Task<()>;

    fn spawn_local<F: std::future::Future<Output = ()> + 'static>(&self, fut: F) -> Self::Task {
        spawn_local(fut)
    }
}
