use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    task::{Context, Waker},
};

use futures::FutureExt;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::Window;

thread_local! {
    static RUNTIME: WebRuntime = WebRuntime::new()
}

pub struct WebSpawner;

struct WebRuntime {
    waker: Waker,
    future: RefCell<Pin<Box<dyn Future<Output = ()>>>>,
    scheduled: Cell<bool>,
    active: Cell<bool>,
    window: Window,
}

impl WebRuntime {
    fn new() -> Self {
        let future = RefCell::new(Box::pin(async {}) as Pin<Box<dyn Future<Output = ()>>>);
        let waker = waker_fn::waker_fn(move || WebSpawner::schedule_now());
        Self {
            waker,
            future,
            scheduled: Cell::new(false),
            active: Cell::new(false),
            window: web_sys::window().expect_throw("failed to get window object"),
        }
    }
}

impl WebSpawner {
    pub(crate) fn set_future<F: Future<Output = ()> + 'static>(future: F) {
        RUNTIME.with(|rt| {
            *rt.future.borrow_mut() = Box::pin(future) as Pin<Box<dyn Future<Output = ()>>>;
        });
    }
    pub fn wake_now() {
        RUNTIME.with(|e| {
            e.scheduled.set(false);
            if !e.active.replace(true) {
                let mut cx = Context::from_waker(&e.waker);
                let _ = e.future.borrow_mut().poll_unpin(&mut cx);
                e.active.set(false);
            }
        })
    }
    pub fn schedule_now() {
        RUNTIME.with(|e| {
            if !e.active.get() && !e.scheduled.replace(true) {
                let closure = Closure::once_into_js(Self::wake_now);
                e.window
                    .set_timeout_with_callback(&closure.as_ref().unchecked_ref())
                    .expect_throw("failed to schedule task");
            }
        })
    }
}
