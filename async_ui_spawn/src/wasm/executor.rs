use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use waker_fn::waker_fn;
use wasm_bindgen::{closure::Closure, JsCast};

use super::SpawnJob;

thread_local! {
    static WEB_EXECUTOR: WebExecutor = WebExecutor::new();
}
scoped_tls::scoped_thread_local! {
    static ACTIVE: bool
}
pub(super) struct WebExecutor {
    tasks: RefCell<HashMap<usize, SpawnJob>>,
    queue: RefCell<VecDeque<usize>>,
    counter: Cell<usize>,
    scheduled: Cell<bool>,
}
impl WebExecutor {
    fn new() -> Self {
        Self {
            tasks: RefCell::new(HashMap::with_capacity(64)),
            queue: RefCell::new(VecDeque::with_capacity(8)),
            counter: Cell::new(0),
            scheduled: Cell::new(false),
        }
    }
    fn spawn(&self, fut: SpawnJob) -> Task {
        let key = self.counter.get();
        self.counter.set(key + 1);
        {
            self.tasks.borrow_mut().insert(key, fut);
        }
        self.enqueue(key);
        Task { key }
    }
    fn poll_task(&self, key: usize) {
        if let Some(mut fut) = {
            let mut bm = self.tasks.borrow_mut();
            bm.remove(&key)
        } {
            let pinned = Pin::new(&mut fut);
            let waker = waker_fn(move || {
                WEB_EXECUTOR.with(|exe| {
                    exe.enqueue(key);
                })
            });
            let mut cx = Context::from_waker(&waker);
            if let Poll::Pending = pinned.poll(&mut cx) {
                self.tasks.borrow_mut().insert(key, fut);
            }
        }
    }
    fn enqueue(&self, key: usize) {
        self.queue.borrow_mut().push_back(key);
        self.schedule();
    }
    fn schedule(&self) {
        if !ACTIVE.is_set() && !self.scheduled.get() {
            if let Some(window) = web_sys::window() {
                let closure = Closure::once(start_executor);
                window
                    .set_timeout_with_callback(&closure.as_ref().unchecked_ref())
                    .ok();
                self.scheduled.set(true);
                closure.forget();
            }
        }
    }
    fn run_queued(&self) {
        if !ACTIVE.is_set() {
            self.scheduled.set(false);
            ACTIVE.set(&true, || {
                while let Some(key) = {
                    let mut bm = self.queue.borrow_mut();
                    bm.pop_front()
                } {
                    self.poll_task(key);
                }
            })
        }
    }
    fn drop_task(&self, key: usize) {
        let task = {
            let mut bm = self.tasks.borrow_mut();
            bm.remove(&key)
        };
        drop(task);
    }
}
pub(super) fn spawn(fut: SpawnJob) -> Task {
    WEB_EXECUTOR.with(|exe| exe.spawn(fut))
}
pub fn start_executor() {
    WEB_EXECUTOR.with(|exe| exe.run_queued());
}
pub struct Task {
    key: usize,
}
impl Drop for Task {
    fn drop(&mut self) {
        WEB_EXECUTOR.with(|exe| exe.drop_task(self.key))
    }
}
