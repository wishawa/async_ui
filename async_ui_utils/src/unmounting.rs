use std::{
    future::Future,
    task::{Context, Poll},
};

pub use async_ui_spawn::is_unmounting;

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
