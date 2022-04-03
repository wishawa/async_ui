use async_executor::{LocalExecutor, Task};
use async_ui_core::local::backend::Spawner;
use glib::MainContext;
use std::{future::Future, rc::Rc};

thread_local! {
    static EXECUTOR: GtkSpawnerInner = GtkSpawnerInner::new();
}
pub struct GtkSpawner;
unsafe impl Spawner for GtkSpawner {
    type Task = Task<()>;

    fn spawn<'a, F: Future<Output = ()> + 'static>(future: F) -> Self::Task {
        EXECUTOR.with(|exe| exe.executor.spawn(future))
    }
}
struct GtkSpawnerInner {
    executor: Rc<LocalExecutor<'static>>,
}
impl GtkSpawnerInner {
    fn new() -> Self {
        let executor = Rc::new(LocalExecutor::new());
        let executor_cpy = executor.clone();
        MainContext::default().spawn_local(async move {
            loop {
                executor_cpy.tick().await;
            }
        });
        Self { executor }
    }
}
