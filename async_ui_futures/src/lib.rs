use std::{future::Future, task::Poll};

pub struct PendForever;
impl Future for PendForever {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Poll::Pending
    }
}
