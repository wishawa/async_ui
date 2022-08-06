use std::{future::Future, pin::Pin, rc::Rc, task::Poll};

use pin_cell::{PinCell, PinMut};
use pin_project_lite::pin_project;

use crate::GiveUnforgettableScope;

pin_project! {
    #[project = InnerProject]
    pub(crate) enum Inner<F: Future> {
        Running {
            #[pin] fut: GiveUnforgettableScope<F>,
        },
        Aborted
    }
}

impl<F: Future> Future for Inner<F> {
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
        remote: Pin<Rc<PinCell<Inner<F>>>>,
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
pub(crate) trait InnerTrait {
    fn abort(self: Pin<&Self>);
}
impl<F: Future> InnerTrait for PinCell<Inner<F>> {
    fn abort(self: Pin<&Self>) {
        let mut bm = self.borrow_mut();
        let mut bm = PinMut::as_mut(&mut bm);
        bm.set(Inner::Aborted);
    }
}
