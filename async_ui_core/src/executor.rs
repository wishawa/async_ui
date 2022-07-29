use std::{future::Future, rc::Rc};

use async_executor::{LocalExecutor, Task};
thread_local! {
    pub(crate) static EXECUTOR: Rc<LocalExecutor<'static>> = {
        let exe= LocalExecutor::new();
        Rc::new(exe)
    }
}
pub(crate) fn spawn_local<F: Future + 'static>(fut: F) -> Task<F::Output> {
    EXECUTOR.with(|exe| exe.spawn(fut))
}

pub(crate) fn get_driving_future() -> impl Future<Output = ()> + 'static {
    EXECUTOR.with(|exe| {
        use async_ui_futures::PendForever;
        let exe_cpy = exe.clone();
        async move { exe_cpy.run(PendForever).await }
    })
}
