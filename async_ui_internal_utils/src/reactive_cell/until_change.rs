use std::future::Future;

use super::ReactiveCell;

impl<T> ReactiveCell<T> {
    pub fn until_change(&'_ self) -> UntilChangeFuture<'_, T> {
        UntilChangeFuture {
            target: self,
            last_version: 0,
        }
    }
}

pub struct UntilChangeFuture<'a, T> {
    target: &'a ReactiveCell<T>,
    last_version: u64,
}

impl<'a, T> Future for UntilChangeFuture<'a, T> {
    type Output = ();
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        let mut bm = this.target.inner.borrow_mut();
        if this.last_version == 0 {
            this.last_version = bm.version;
            bm.listeners.push(cx.waker().to_owned());
            std::task::Poll::Pending
        } else if this.last_version != bm.version {
            std::task::Poll::Ready(())
        } else {
            std::task::Poll::Pending
        }
    }
}
