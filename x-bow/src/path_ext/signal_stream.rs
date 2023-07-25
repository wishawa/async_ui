use std::{cell::Ref, pin::Pin, task::Poll};

use futures_core::Stream;

use crate::{until_change::UntilChange, Path, PathExt};

/// A [Stream]. See [signal_stream][crate::PathExt::signal_stream].
pub struct SignalStream<'a, P: Path + ?Sized> {
    path: &'a P,
    until_change: UntilChange<'a>,
    fire_immediately: bool,
}

impl<'a, P: Path + ?Sized> SignalStream<'a, P> {
    pub(super) fn new(path: &'a P, until_change: UntilChange<'a>, fire_immediately: bool) -> Self {
        Self {
            path,
            until_change,
            fire_immediately,
        }
    }
}

impl<'a, P: Path + ?Sized> Stream for SignalStream<'a, P> {
    type Item = Ref<'a, P::Out>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let first = !this.until_change.has_been_polled();
        if (first && this.fire_immediately)
            | Pin::new(&mut this.until_change).poll_next(cx).is_ready()
        {
            Poll::Ready(this.path.borrow_opt())
        } else {
            Poll::Pending
        }
    }
}
