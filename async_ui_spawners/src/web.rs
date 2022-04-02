use std::{cell::Cell, future::Future, pin::Pin, task::Context};

use async_executor::{LocalExecutor, Task};
use async_ui_core::backend::Spawner;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::Window;

thread_local! {
    static EXECUTOR: WebSpawnerInner = WebSpawnerInner::new()
}

pub struct WebSpawner;

struct WebSpawnerInner {
    executor: LocalExecutor<'static>,
    scheduled: Cell<bool>,
    active: Cell<bool>,
    window: Window,
}

impl WebSpawnerInner {
    fn new() -> Self {
        Self {
            executor: LocalExecutor::new(),
            scheduled: Cell::new(false),
            active: Cell::new(false),
            window: web_sys::window().expect_throw("failed to get window object"),
        }
    }
}

unsafe impl Spawner for WebSpawner {
    type Task = Task<()>;

    fn spawn<'a, F: Future<Output = ()> + 'static>(future: F) -> Self::Task {
        let task = EXECUTOR.with(|e| e.executor.spawn(WakerHookWrap { future }));
        task
    }

    fn wake_now() {
        EXECUTOR.with(|e| {
            e.scheduled.set(false);
            if !e.active.replace(true) {
                while e.executor.try_tick() {}
                e.active.set(false);
            }
        })
    }

    fn schedule_now() {
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

pin_project_lite::pin_project! {
    struct WakerHookWrap<F: Future> {
        #[pin]
        future: F
    }
}
impl<F: Future> Future for WakerHookWrap<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let waker = cx.waker().to_owned();
        let new_waker = waker_fn::waker_fn(move || {
            waker.wake_by_ref();
            WebSpawner::schedule_now();
        });
        let mut new_cx = Context::from_waker(&new_waker);
        this.future.poll(&mut new_cx)
    }
}
