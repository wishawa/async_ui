use std::{
    future::Future,
    task::{Context, Poll},
};

use crate::shared::UNMOUNTING;

pub fn is_unmounting() -> bool {
    UNMOUNTING.is_set()
}

pub struct UntilUnmountFuture;
pub fn until_unmount() -> UntilUnmountFuture {
    UntilUnmountFuture
}
impl Future for UntilUnmountFuture {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match is_unmounting() {
            true => Poll::Ready(()),
            false => Poll::Pending,
        }
    }
}
