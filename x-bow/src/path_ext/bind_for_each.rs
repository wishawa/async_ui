use std::{future::Future, task::Poll};

use futures_core::Stream;

use crate::{until_change::UntilChange, Path, PathExt};

pin_project_lite::pin_project! {
    /// A [Future]. See [bind_for_each][crate::PathExt::bind_for_each].
    pub struct BindForEach<'a, P: ?Sized, C, I>
    {
        path: &'a P,
        #[pin]
        until_change: UntilChange<'a>,
        closure: C,
        #[pin]
        incoming: I
    }
}

impl<'a, P: ?Sized, C, I> BindForEach<'a, P, C, I> {
    pub(super) fn new(path: &'a P, until_change: UntilChange<'a>, closure: C, incoming: I) -> Self {
        Self {
            path,
            until_change,
            closure,
            incoming,
        }
    }
}

impl<'a, P: Path + ?Sized, C: FnMut(&P::Out) + Unpin, I: Stream<Item = P::Out>> Future
    for BindForEach<'a, P, C, I>
where
    P::Out: Sized,
{
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        let mut val = None;
        loop {
            match this.incoming.as_mut().poll_next(cx) {
                Poll::Ready(Some(v)) => {
                    val = Some(v);
                }
                Poll::Ready(None) => {
                    return Poll::Ready(());
                }
                Poll::Pending => break,
            }
        }
        if let Some(val) = val {
            if let Some(bm) = this.path.borrow_opt_mut().as_deref_mut() {
                *bm = val;
            } else {
                return Poll::Ready(());
            }
            let _ = this.until_change.as_mut().poll_next(cx);
        }

        let first = !this.until_change.has_been_polled();
        if first | this.until_change.as_mut().poll_next(cx).is_ready() {
            if let Some(data) = this.path.borrow_opt().as_deref() {
                (this.closure)(data);
            }
        }
        Poll::Pending
    }
}
