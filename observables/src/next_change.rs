use std::{
    borrow::Borrow,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::{ObservableBase, Version};

pin_project! {
    pub struct NextChangeFuture<I, A>
    where
        A: Borrow<I>,
        I: ObservableBase,
        I: ?Sized,
    {
        inner: A,
        start_version: Version,
        _phantom: PhantomData<Box<I>>,
    }
}
impl<I, A> NextChangeFuture<I, A>
where
    A: Borrow<I>,
    I: ObservableBase,
    I: ?Sized,
{
    pub fn new(observable: A) -> Self {
        Self {
            inner: observable,
            start_version: Version::new_null(),
            _phantom: PhantomData,
        }
    }
    pub fn observable(&self) -> &A {
        &self.inner
    }
    pub fn rewind(&mut self) {
        self.start_version = Version::new_null();
    }
}

impl<I, A> Future for NextChangeFuture<I, A>
where
    A: Borrow<I>,
    I: ObservableBase,
    I: ?Sized,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let inner: &I = (&*this.inner).borrow();
        if this.start_version.is_null() {
            *this.start_version = inner.get_version();
            inner.add_waker(cx.waker().to_owned());
        }
        if inner.get_version() > *this.start_version {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
