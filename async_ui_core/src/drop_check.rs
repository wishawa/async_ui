use std::{future::Future, pin::Pin};

scoped_tls::scoped_thread_local!(
    static DROP_SCOPE: (*const (), *const ())
);

pub struct PropagateDropScope<F: ?Sized + Future<Output = ()>> {
    future: Pin<Box<F>>,
}

impl<F: ?Sized + Future<Output = ()>> PropagateDropScope<F> {
    pub fn new(future: Pin<Box<F>>) -> Self {
        Self { future }
    }
}
impl<F: ?Sized + Future<Output = ()>> Future for PropagateDropScope<F> {
    type Output = ();

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let target = &*self.future;
        let target_start = target as *const F as *const () as usize;
        let size = std::mem::size_of_val(target);
        let target_end = target_start + size;
        DROP_SCOPE.set(
            &(target_start as *const (), target_end as *const ()),
            || self.future.as_mut().poll(cx),
        )
    }
}

pub fn check_drop_scope(ptr: *const ()) {
    DROP_SCOPE.with(|&(low, high)| {
        if low > ptr || high <= ptr {
            panic!("drop scope violated");
        }
    })
}
