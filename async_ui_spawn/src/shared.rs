use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use scoped_tls::ScopedKey;

scoped_tls::scoped_thread_local!(
    pub(crate) static DROP_GUARANTEED_SCOPED: (*const (), *const ())
);
pub trait SpawnContext: Sized + 'static {
    fn get_tls() -> &'static ScopedKey<Self>;
}

pin_project_lite::pin_project! {
    pub(crate) struct SpawnWrappedFuture<F, C>
    where F: ?Sized, F: 'static, C: SpawnContext
    {
        future: Pin<Box<F>>,
        context: C,
    }
}

impl<F: ?Sized, C: SpawnContext> SpawnWrappedFuture<F, C> {
    pub fn new(future: Pin<Box<F>>, context: C) -> Self {
        Self { future, context }
    }
}

impl<'a, F: ?Sized + Future + 'static, C: SpawnContext> Future for SpawnWrappedFuture<F, C> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let target = &**this.future;
        let target_start = target as *const _ as *const ();
        let size = std::mem::size_of_val(target);
        let target_end = target_start.wrapping_add(size);
        DROP_GUARANTEED_SCOPED.set(&(target_start, target_end), || {
            C::get_tls().set(this.context, || this.future.as_mut().poll(cx))
        })
    }
}

pin_project_lite::pin_project! {
    pub struct RootSpawnWrapperFuture<F>
    where F: Future, F: 'static
    {
        #[pin]
        future: F,
    }
}

impl<F: Future + 'static> Future for RootSpawnWrapperFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let target_start = &*this.future as *const F as *const ();
        let target_end = target_start.wrapping_add(std::mem::size_of::<F>());
        DROP_GUARANTEED_SCOPED.set(&(target_start, target_end), || this.future.poll(cx))
    }
}

impl<F: Future + 'static> RootSpawnWrapperFuture<F> {
    pub fn new(future: F) -> Self {
        Self { future }
    }
}
