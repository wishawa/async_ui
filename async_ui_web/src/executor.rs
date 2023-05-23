use std::{cell::Cell, future::pending};

use async_executor::LocalExecutor;
use async_ui_web_core::executor::set_executor_future;

thread_local! {
    static EXECUTOR: Cell<Option<&'static LocalExecutor<'static>>> = Cell::new(None);
}

pub fn get_executor() -> &'static LocalExecutor<'static> {
    EXECUTOR.with(|cell| {
        if let Some(exe) = cell.get() {
            exe
        } else {
            let leaked = Box::leak(Box::new(LocalExecutor::new()));
            cell.set(Some(&*leaked));
            set_executor_future(Box::new(leaked.run(pending())));
            &*leaked
        }
    })
}
