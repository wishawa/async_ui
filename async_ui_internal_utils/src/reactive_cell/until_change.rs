use std::{future::Future, task::Poll};

use futures_core::Stream;

use super::ReactiveCell;

impl<T> ReactiveCell<T> {
    pub fn until_change(&'_ self) -> UntilChangeFuture<'_, T> {
        UntilChangeFuture {
            target: self,
            last_version: 0,
            waker_idx: usize::MAX,
        }
    }
}

pub struct UntilChangeFuture<'a, T> {
    target: &'a ReactiveCell<T>,
    last_version: u64,
    waker_idx: usize,
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
        let new = cx.waker();
        match bm.listeners.get_mut(this.waker_idx) {
            Some(existing) if existing.will_wake(new) => {}
            _ => {
                this.waker_idx = bm.listeners.len();
                bm.listeners.push(new.to_owned());
            }
        }
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
