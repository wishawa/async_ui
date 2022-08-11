use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    task::{Context, Waker},
};

use futures::FutureExt;
use wasm_bindgen::{closure::Closure, JsCast, UnwrapThrowExt};

use crate::window::WINDOW;

thread_local! {
    static EXECUTOR: ExecutorSingleton = ExecutorSingleton::new()
}
struct ExecutorSingleton {
    waker: Waker,
    future: RefCell<Option<Pin<Box<dyn Future<Output = ()>>>>>,
    scheduled: Cell<bool>,
    active: Cell<bool>,
}

impl ExecutorSingleton {
    fn new() -> Self {
        let waker = waker_fn::waker_fn(schedule);
        Self {
            waker,
            future: RefCell::new(None),
            scheduled: Cell::new(false),
            active: Cell::new(false),
        }
    }
}
pub(crate) fn set_executor_future(future: Box<dyn Future<Output = ()>>) {
    EXECUTOR.with(|exe| *exe.future.borrow_mut() = Some(future.into()))
}
pub fn run_now() {
    EXECUTOR.with(|exe| {
        exe.active.set(true);
        while exe.scheduled.replace(false) {
            let mut cx = Context::from_waker(&exe.waker);
            match exe.future.borrow_mut().as_mut() {
                Some(fu) => {
                    let _ = fu.as_mut().poll_unpin(&mut cx);
                }
                None => {}
            }
        }
        exe.active.set(false);
    })
}
pub fn schedule() {
    EXECUTOR.with(|exe| {
        if !exe.scheduled.replace(true) && !exe.active.get() {
            let closure = Closure::once_into_js(run_now);
            WINDOW.with(|window| {
                window
                    .set_timeout_with_callback(&closure.as_ref().unchecked_ref())
                    .expect_throw("failed to schedule task");
            })
        }
    })
}
