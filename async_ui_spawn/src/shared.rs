use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

scoped_tls::scoped_thread_local!(
    pub(crate) static DROP_GUARANTEED_SCOPED: (*const (), *const ())
);
scoped_tls::scoped_thread_local!(
    pub(crate) static UNMOUNTING: bool
);

pub(crate) struct SpawnWrappedFuture<F>
where
    F: ?Sized + Future + 'static,
{
    future: Pin<Box<F>>,
}

impl<F: ?Sized + Future + 'static> SpawnWrappedFuture<F> {
    pub fn new(future: Pin<Box<F>>) -> Self {
        Self { future }
    }
}
thread_local! {
    static DUMMY_WAKER: Waker = waker_fn::waker_fn(|| {})
}
impl<F: ?Sized + Future + 'static> Drop for SpawnWrappedFuture<F> {
    fn drop(&mut self) {
        DUMMY_WAKER.with(|wk| {
            let mut cx = Context::from_waker(wk);
            web_sys::console::log_1(&"last poll!".into());
            UNMOUNTING.set(&true, || {
                let _ = self.future.as_mut().poll(&mut cx);
            });
        });
    }
}

impl<'a, F: ?Sized + Future + 'static> Future for SpawnWrappedFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self;
        let target = &*this.future;
        let target_start = target as *const F as *const () as usize;
        let size = std::mem::size_of_val(target);
        let target_end = target_start + size;
        DROP_GUARANTEED_SCOPED.set(
            &(target_start as *const (), target_end as *const ()),
            || this.future.as_mut().poll(cx),
        )
    }
}

pin_project_lite::pin_project! {
    pub struct RootSpawnWrappedFuture<F>
    where F: Future, F: 'static
    {
        #[pin]
        future: F,
    }
}

impl<F: Future + 'static> Future for RootSpawnWrappedFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let target_start = &*this.future as *const F;
        let target_end = target_start.wrapping_add(1);
        DROP_GUARANTEED_SCOPED.set(
            &(target_start as *const (), target_end as *const ()),
            || this.future.poll(cx),
        )
    }
}

impl<F: Future + 'static> RootSpawnWrappedFuture<F> {
    pub fn new(future: F) -> Self {
        Self { future }
    }
}

pub(crate) fn check_drop_guarantee<T>(ptr: &Pin<&mut T>) {
    let self_loc = &**ptr as *const T as *const ();
    DROP_GUARANTEED_SCOPED.with(|&(low, high)| {
        if low > self_loc || high <= self_loc {
            panic!("drop guarantee violated");
        }
    });
}
