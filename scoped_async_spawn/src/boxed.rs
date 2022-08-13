use std::{future::Future, marker::PhantomPinned, ops::Deref, pin::Pin, task::Poll};

use pin_project_lite::pin_project;

use crate::{
    pointer::Pointer,
    scope::{check_scope, set_scope},
};

pin_project! {
    pub struct ScopeSafeBox<T>
    where
        T: ?Sized
    {
        #[pin]
        _phantom: PhantomPinned,
        data: Pin<Box<T>>,
    }
}
impl<T: ?Sized> Deref for ScopeSafeBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.data
    }
}
impl<T: ?Sized> ScopeSafeBox<T> {
    pub fn from_boxed(boxed: Box<T>) -> Self {
        let data = boxed.into();
        Self {
            _phantom: PhantomPinned,
            data,
        }
    }
    pub fn with_scope<R, F: FnOnce(Pin<&mut T>) -> R>(self: Pin<&mut Self>, func: F) -> R {
        let this = self.project();
        let fut_ref: &T = &**this.data;
        let fut_ptr = Pointer::new(fut_ref);
        unsafe { set_scope(fut_ptr, || func(this.data.as_mut())) }
    }
}
impl<F: Future + ?Sized> Future for ScopeSafeBox<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let here = Pointer::new(&*self);
        let is_in_scope = check_scope(here);
        if !is_in_scope {
            panic!("Not in scope.");
        } else {
            self.with_scope(|fut| fut.poll(cx))
        }
    }
}
