use std::{future::pending, rc::Rc};

use async_executor::{LocalExecutor, Task};
use futures::Future;

pub(crate) struct Runtime {
    executor: Rc<LocalExecutor<'static>>,
}
thread_local! {
    pub(crate) static RUNTIME: Runtime = Runtime {
        executor: Rc::new(LocalExecutor::new())
    };
}

impl Runtime {
    pub fn spawn<F: Future + 'static>(&self, future: F) -> Task<F::Output> {
        self.executor.spawn(future)
    }
}
pub async fn drive_runtime() {
    let exe_cpy = RUNTIME.with(|rt| rt.executor.clone());
    exe_cpy.run(pending()).await
}
