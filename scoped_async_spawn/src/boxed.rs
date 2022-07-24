use std::{future::Future, marker::PhantomPinned, pin::Pin, task::Poll};

use pin_project_lite::pin_project;

use crate::{pointer::Pointer, UNFORGETTABLE_SCOPE};

pin_project! {
    pub struct ScopeSafeBox<T> {
        data: Pin<Box<T>>,
        _phantom: PhantomPinned
    }
}
impl<F: Future> Future for ScopeSafeBox<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let here = Pointer::new(&*self);
        let is_in_scope = UNFORGETTABLE_SCOPE.with(|sc| sc.contains(here));
        if !is_in_scope {
            panic!("Not in scope.");
        } else {
            let this = self.project();
            let fut_ref: &F = &**this.data;
            let fut_ptr = Pointer::new(fut_ref);
            UNFORGETTABLE_SCOPE.set(&fut_ptr, || this.data.as_mut().poll(cx))
        }
    }
}
