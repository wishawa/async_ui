use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Waker},
};

use async_executor::{LocalExecutor, Task};
use futures::FutureExt;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::Window;

thread_local! {
    static EXECUTOR: WebSpawnerInner = WebSpawnerInner::new()
}

pub struct WebSpawner;

struct WebSpawnerInner {
    executor: Rc<LocalExecutor<'static>>,
    waker: Waker,
    future: RefCell<Pin<Box<dyn Future<Output = ()>>>>,
    scheduled: Cell<bool>,
    active: Cell<bool>,
    window: Window,
}

impl WebSpawnerInner {
    fn new() -> Self {
        let executor = Rc::new(LocalExecutor::new());
        let executor_cpy = executor.clone();
        let future = async move {
            loop {
                executor_cpy.tick().await
            }
        };
        let future = RefCell::new(Box::pin(future) as Pin<Box<_>>);
        let waker = waker_fn::waker_fn(move || WebSpawner::schedule_now());
        Self {
            executor,
            waker,
            future,
            scheduled: Cell::new(false),
            active: Cell::new(false),
            window: web_sys::window().expect_throw("failed to get window object"),
        }
    }
}

impl WebSpawner {
    pub fn wake_now() {
        EXECUTOR.with(|e| {
            e.scheduled.set(false);
            if !e.active.replace(true) {
                let mut cx = Context::from_waker(&e.waker);
                let _ = e.future.borrow_mut().poll_unpin(&mut cx);
                e.active.set(false);
            }
        })
    }

    pub fn schedule_now() {
        EXECUTOR.with(|e| {
            if !e.active.get() && !e.scheduled.replace(true) {
                let closure = Closure::once_into_js(Self::wake_now);
                e.window
                    .set_timeout_with_callback(&closure.as_ref().unchecked_ref())
                    .expect_throw("failed to schedule task");
            }
        })
    }
}
