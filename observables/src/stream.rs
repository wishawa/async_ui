use std::{pin::Pin, task::Poll};

use futures::{Future, Stream};

use crate::Observable;

pub struct ObserveStream<'a, T> {
    observable: &'a Observable<T>,
    key: Option<usize>,
    version: usize,
}
impl<'a, T> Stream for ObserveStream<'a, T> {
    type Item = ();
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.key.is_none() {
            let mut bm = this.observable.inner.borrow_mut();
            let key = bm.listeners.insert(cx.waker().to_owned());
            this.key = Some(key);
        }
        let latest = this.observable.inner.borrow().version;
        if this.version < latest {
            this.version = latest;
            Poll::Ready(Some(()))
        } else {
            Poll::Pending
        }
    }
}
impl<'a, T> Drop for ObserveStream<'a, T> {
    fn drop(&mut self) {
        if let Some(key) = self.key {
            self.observable.inner.borrow_mut().listeners.remove(key);
        }
    }
}
pub struct ObserveOnce<'a, T> {
    stream: ObserveStream<'a, T>,
}
impl<'a, T> Future for ObserveOnce<'a, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        use futures::StreamExt;
        self.get_mut().stream.poll_next_unpin(cx).map(|_| ())
    }
}
impl<T> Observable<T> {
    pub fn stream_change(&self) -> ObserveStream<'_, T> {
        ObserveStream {
            observable: self,
            key: None,
            version: self.inner.borrow().version,
        }
    }
    pub fn until_next_change(&self) -> ObserveOnce<'_, T> {
        ObserveOnce {
            stream: self.stream_change(),
        }
    }
}
