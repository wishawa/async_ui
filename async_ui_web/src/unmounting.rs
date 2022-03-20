use std::{
    future::Future,
    task::{Context, Poll},
};

scoped_tls::scoped_thread_local! {
    pub(crate) static UNMOUNTING: bool
}
pub struct UntilUnmountFuture;
impl Future for UntilUnmountFuture {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        UNMOUNTING.with(|unmounting| {
            if *unmounting {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
    }
}
