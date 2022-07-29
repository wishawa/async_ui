use std::{future::Future, pin::Pin, task::Poll};

use super::pointer::Pointer;
use pin_project_lite::pin_project;
use scoped_tls::scoped_thread_local;

scoped_thread_local!(static UNFORGETTABLE_SCOPE: Pointer);
pin_project! {
    pub struct GiveUnforgettableScope<F: Future> {
        #[pin] fut: F,
    }
}
impl<F: Future> Future for GiveUnforgettableScope<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let fut_ref: &F = &self.as_ref().fut;
        let fut_ptr = Pointer::new(fut_ref);
        UNFORGETTABLE_SCOPE.set(&fut_ptr, || self.project().fut.poll(cx))
    }
}
impl<F: Future + 'static> GiveUnforgettableScope<F> {
    pub fn new_static(fut: F) -> Self {
        unsafe { Self::new(fut) }
    }
}
impl<F: Future> GiveUnforgettableScope<F> {
    pub unsafe fn new(fut: F) -> Self {
        Self { fut }
    }
}
pub fn check_scope(here: Pointer) -> bool {
    UNFORGETTABLE_SCOPE.is_set() && UNFORGETTABLE_SCOPE.with(|sc| sc.contains(here))
}
pub unsafe fn set_scope<R, F: FnOnce() -> R>(val: Pointer, f: F) -> R {
    UNFORGETTABLE_SCOPE.set(&val, f)
}
