use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
pub use scoped_tls::{scoped_thread_local, ScopedKey};

pin_project! {
    pub struct GiveContext<'a, T: 'static, F> {
        #[pin]
        future: F,
        key: &'static ScopedKey<T>,
        data: &'a T,
    }
}

impl<'a, T, F> Future for GiveContext<'a, T, F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.key.set(*this.data, || this.future.poll(cx))
    }
}

pub trait ScopedTlsAsync<T> {
    fn set_async<'a, F: Future + 'a>(
        &'static self,
        data: &'a T,
        future: F,
    ) -> GiveContext<'a, T, F>;
}
impl<T> ScopedTlsAsync<T> for ScopedKey<T> {
    fn set_async<'a, F: Future + 'a>(
        &'static self,
        data: &'a T,
        future: F,
    ) -> GiveContext<'a, T, F> {
        GiveContext {
            future,
            key: self,
            data,
        }
    }
}
