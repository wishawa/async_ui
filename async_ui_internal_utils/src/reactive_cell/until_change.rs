use std::{future::Future, task::Poll};

use futures_core::Stream;

use crate::wakers_list::WakerSlot;

use super::{ReactiveCell, SUBLIST};

impl<T> ReactiveCell<T> {
    pub fn until_change(&'_ self) -> UntilChangeFuture<'_, T> {
        UntilChangeFuture {
            target: self,
            last_version: 0,
            waker_slot: self.inner.borrow_mut().listeners.add(&SUBLIST),
        }
    }
}

pub struct UntilChangeFuture<'a, T> {
    target: &'a ReactiveCell<T>,
    last_version: u64,
    waker_slot: WakerSlot,
}

impl<'a, T> Stream for UntilChangeFuture<'a, T> {
    type Item = ();

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut bm = this.target.inner.borrow_mut();
        let mut res = Poll::Pending;
        match this.last_version {
            0 => this.last_version = bm.version,
            lv if lv < bm.version => {
                res = Poll::Ready(Some(()));
                this.last_version = bm.version;
            }
            _ => {}
        }
        bm.listeners.update(&this.waker_slot, cx.waker());
        res
    }
}

impl<'a, T> Future for UntilChangeFuture<'a, T> {
    type Output = ();
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.poll_next(cx).map(|_| ())
    }
}

impl<'a, T> Drop for UntilChangeFuture<'a, T> {
    fn drop(&mut self) {
        self.target
            .inner
            .borrow_mut()
            .listeners
            .remove(&self.waker_slot);
    }
}
