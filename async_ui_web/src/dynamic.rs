use std::{
    cell::RefCell,
    future::{poll_fn, Future},
    pin::pin,
    task::{Poll, Waker},
};

use async_ui_internal_utils::dummy_waker::dummy_waker;

enum Next<F> {
    Set(F),
    Clear,
    NoChange,
}

pub struct Dynamic<F: Future> {
    next: RefCell<(Waker, Next<F>)>,
}

impl<F: Future> Dynamic<F> {
    pub fn new() -> Self {
        Self {
            next: RefCell::new((dummy_waker(), Next::NoChange)),
        }
    }

    pub async fn render(&self) {
        let mut fut = pin!(None);
        poll_fn(|cx| {
            {
                let mut next = self.next.borrow_mut();
                match std::mem::replace(&mut next.1, Next::NoChange) {
                    Next::Set(new_fut) => fut.set(Some(new_fut)),
                    Next::Clear => fut.set(None),
                    Next::NoChange => {}
                }
                if !next.0.will_wake(cx.waker()) {
                    next.0 = cx.waker().to_owned();
                }
            }
            if let Some(fut) = fut.as_mut().as_pin_mut() {
                let _ = fut.poll(cx);
            }
            Poll::Pending
        })
        .await
    }
    pub fn set_future(&self, fut: F) {
        let mut bm = self.next.borrow_mut();
        bm.1 = Next::Set(fut);
        bm.0.wake_by_ref();
    }
    pub fn clear_future(&self) {
        let mut bm = self.next.borrow_mut();
        bm.1 = Next::Clear;
        bm.0.wake_by_ref();
    }
}
