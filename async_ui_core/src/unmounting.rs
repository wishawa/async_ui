use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
scoped_tls::scoped_thread_local!(
    pub(crate) static UNMOUNTING: bool
);

#[must_use = "UntilUnmount is a Future and should be awaited"]
pub struct UntilUnmount;

impl Future for UntilUnmount {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if UNMOUNTING.with(Clone::clone) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
