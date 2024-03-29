//! Exposes a [LocalExecutor] for general use.
//!
//! Async UI Web creates an app-wide executor
//! Use the [get_executor] function to access the executor.

use std::{cell::OnceCell, future::pending};

use async_executor::LocalExecutor;
use async_ui_web_core::executor::set_executor_future;

thread_local! {
    static EXECUTOR: OnceCell<&'static LocalExecutor<'static>> = OnceCell::new();
}

/// Get the executor that is driving the framework.
/// Use this executor to spawn your own tasks if you want.
///
/// ```
/// # use async_ui_web::executor::get_executor;
/// # async fn some_async_function() {}
/// # fn example() {
/// let exe = get_executor();
/// let task = exe.spawn(some_async_function());
/// # }
/// ```
pub fn get_executor() -> &'static LocalExecutor<'static> {
    EXECUTOR.with(|cell| {
        *cell.get_or_init(|| {
            let exe = Box::leak(Box::new(LocalExecutor::new()));
            set_executor_future(Box::new(exe.run(pending())));
            exe
        })
    })
}
