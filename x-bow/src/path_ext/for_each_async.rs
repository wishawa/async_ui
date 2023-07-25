use std::{future::Future, pin::Pin, task::Poll};

use futures_core::Stream;

use crate::{until_change::UntilChange, Path, PathExt};

pin_project_lite::pin_project! {
    /// A [Future]. See [for_each][crate::PathExt::for_each].
    pub struct ForEachAsync<'a, P, F: Future<Output = ()>, C: FnMut(&P::Out) -> F>
    where
        P: Path,
        P: ?Sized
    {
        path: &'a P,
        until_change: UntilChange<'a>,
        closure: C,
        #[pin]
        future: Option<F>
    }
}

impl<'a, P: Path + ?Sized, F: Future<Output = ()>, C: FnMut(&P::Out) -> F>
    ForEachAsync<'a, P, F, C>
{
    pub(super) fn new(path: &'a P, until_change: UntilChange<'a>, closure: C) -> Self {
        Self {
            path,
            until_change,
            closure,
            future: None,
        }
    }
}

impl<'a, P: Path + ?Sized, F: Future<Output = ()>, C: FnMut(&P::Out) -> F + Unpin> Future
    for ForEachAsync<'a, P, F, C>
{
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let first = !this.until_change.has_been_polled();
        if first | Pin::new(this.until_change).poll_next(cx).is_ready() {
            if let Some(data) = this.path.borrow_opt().as_deref() {
                let fut = (this.closure)(data);
                this.future.set(Some(fut));
            } else {
                return Poll::Ready(());
            }
        }
        if let Some(fut) = this.future.as_mut().as_pin_mut() {
            if fut.poll(cx).is_ready() {
                this.future.set(None);
            }
        }
        Poll::Pending
    }
}
