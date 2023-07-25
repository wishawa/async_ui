use std::{future::Future, pin::Pin, task::Poll};

use futures_core::Stream;

use crate::{until_change::UntilChange, Path, PathExt};

/// A [Future]. See [for_each][crate::PathExt::for_each].
pub struct ForEach<'a, P: Path + ?Sized, C: FnMut(&P::Out)> {
    path: &'a P,
    until_change: UntilChange<'a>,
    closure: C,
}

impl<'a, P: Path + ?Sized, C: FnMut(&P::Out)> ForEach<'a, P, C> {
    pub(super) fn new(path: &'a P, until_change: UntilChange<'a>, closure: C) -> Self {
        Self {
            path,
            until_change,
            closure,
        }
    }
}

impl<'a, P: Path + ?Sized, C: FnMut(&P::Out) + Unpin> Future for ForEach<'a, P, C> {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let first = !this.until_change.has_been_polled();
        if first | Pin::new(&mut this.until_change).poll_next(cx).is_ready() {
            if let Some(data) = this.path.borrow_opt().as_deref() {
                (this.closure)(data);
            }
        }
        Poll::Pending
    }
}
