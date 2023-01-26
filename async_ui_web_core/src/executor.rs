/*! The executor responsible for schduling and running futures. Not relevant to users.
 *
 */
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

use wasm_bindgen::{closure::Closure, JsCast, UnwrapThrowExt};

use crate::window::WINDOW;

thread_local! {
    static EXECUTOR: ExecutorSingleton = ExecutorSingleton::new();
}
struct ExecutorSingleton {
    waker: Waker,
    future: RefCell<Option<Pin<Box<dyn Future<Output = ()>>>>>,
    scheduled: Cell<bool>,
    active: Cell<bool>,
    run_closure: Closure<dyn Fn()>,
}

impl ExecutorSingleton {
    fn new() -> Self {
        Self {
            waker: root_waker(),
            future: RefCell::new(None),
            scheduled: Cell::new(false),
            active: Cell::new(false),
            run_closure: Closure::new(schedule),
        }
    }
}

/// Set the root future to execute.
/// We usually call this with future returned from `async_executor::LocalExecutor::run(...)`.
pub fn set_executor_future(future: Box<dyn Future<Output = ()>>) {
    EXECUTOR.with(|exe| *exe.future.borrow_mut() = Some(future.into()))
}

/// Run the root executor immediately.
/// Normally, the executor would have to wait one microtask after being woken
/// before it starts polling its future.
/// But by then `.preventDefault()` no longer works.
/// So our event handlers call `run_now` to let the Rust app handle events immediately.
pub fn run_now() {
    EXECUTOR.with(|exe| {
        let was_active = exe.active.replace(true);
        if !was_active {
            while exe.scheduled.replace(false) {
                let mut cx = Context::from_waker(&exe.waker);
                match exe.future.borrow_mut().as_mut() {
                    Some(fu) => {
                        let _ = fu.as_mut().poll(&mut cx);
                    }
                    None => {}
                }
            }
            exe.active.set(false);
        }
    })
}

/// Schedule the executor to poll its future.
/// Does nothing if already scheduled.
/// If not already scheduled, the executor will queue itself to run in the next microtask.
pub fn schedule() {
    EXECUTOR.with(|exe| {
        if !exe.scheduled.replace(true) && !exe.active.get() {
            WINDOW.with(|window| {
                window
                    .set_timeout_with_callback(&exe.run_closure.as_ref().unchecked_ref())
                    .expect_throw("failed to schedule task");
            })
        }
    })
}

/// A waker for the root future. Calls `schedule()` when woke.
pub(crate) fn root_waker() -> Waker {
    fn new_raw_waker() -> RawWaker {
        RawWaker::new(
            core::ptr::null::<()>(),
            &RawWakerVTable::new(|_| new_raw_waker(), |_| schedule(), |_| schedule(), |_| {}),
        )
    }
    unsafe { Waker::from_raw(new_raw_waker()) }
}
