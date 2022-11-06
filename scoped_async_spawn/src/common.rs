use std::{future::Future, pin::Pin, rc::Rc, task::Poll};

use pin_cell::{PinCell, PinMut};
use pin_project_lite::pin_project;

use crate::GiveUnforgettableScope;

pin_project! {
    #[project = InnerProject]
    pub(crate) enum WrappedFuture<F: Future> {
        Running {
            #[pin] fut: GiveUnforgettableScope<F>,
        },
        Aborted
    }
}

impl<F: Future> Future for WrappedFuture<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match this {
            InnerProject::Running { fut } => fut.poll(cx),
            InnerProject::Aborted => Poll::Pending,
        }
    }
}
pub struct RemoteStaticFuture<T> {
    remote: Pin<Rc<PinCell<dyn Future<Output = T> + 'static>>>,
}
impl<T> RemoteStaticFuture<T> {
    pub(crate) unsafe fn new<'x, F: Future<Output = T> + 'x>(
        remote: Pin<Rc<PinCell<WrappedFuture<F>>>>,
    ) -> Self {
        let remote = remote as Pin<Rc<PinCell<dyn Future<Output = T> + 'x>>>;
        let remote = unsafe {
            std::mem::transmute::<
                Pin<Rc<PinCell<dyn Future<Output = T> + 'x>>>,
                Pin<Rc<PinCell<dyn Future<Output = T> + 'static>>>,
            >(remote)
        };
        Self { remote }
    }
}
impl<T> Future for RemoteStaticFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let cell = self.remote.as_ref();
        let mut bm = cell.borrow_mut();
        let bm = PinMut::as_mut(&mut bm);
        bm.poll(cx)
    }
}
pub(crate) trait WrappedFutureTrait {
    fn drop_future_now(self: Pin<&Self>);
}
impl<F: Future> WrappedFutureTrait for PinCell<WrappedFuture<F>> {
    fn drop_future_now(self: Pin<&Self>) {
        let mut bm = self.borrow_mut();
        let mut bm = PinMut::as_mut(&mut bm);
        bm.set(WrappedFuture::Aborted);
    }
}
