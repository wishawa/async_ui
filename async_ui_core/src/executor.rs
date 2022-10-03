use std::{
    future::{pending, Future},
    rc::Rc,
};

use async_executor::{LocalExecutor, Task};
thread_local! {
    pub(crate) static EXECUTOR: Rc<LocalExecutor<'static>> = {
        let exe= LocalExecutor::new();
        Rc::new(exe)
    }
}
pub fn spawn_local<F: Future + 'static>(fut: F) -> Task<F::Output> {
    EXECUTOR.with(|exe| exe.spawn(fut))
}

pub(crate) fn get_driving_future() -> impl Future<Output = ()> + 'static {
    EXECUTOR.with(|exe| {
        let exe_cpy = exe.clone();
        async move { exe_cpy.run(pending()).await }
    })
}
