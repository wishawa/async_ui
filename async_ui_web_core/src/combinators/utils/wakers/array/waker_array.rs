use core::array;
use core::task::Waker;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use super::{
    super::shared_arc::{waker_from_redirect_position, SharedArcContent},
    ReadinessArray,
};

/// A collection of wakers which delegate to an in-line waker.
pub(crate) struct WakerArray<const N: usize> {
    wakers: [Waker; N],
    inner: Arc<WakerArrayInner<N>>,
}

/// See [super::super::shared_arc] for how this works.
struct WakerArrayInner<const N: usize> {
    redirect: [*const Self; N],
    readiness: Mutex<ReadinessArray<N>>,
}

impl<const N: usize> WakerArray<N> {
    /// Create a new instance of `WakerArray`.
    pub(crate) fn new() -> Self {
        let inner = Arc::new_cyclic(|w| {
            // `Weak::as_ptr` on a live Weak gives the same thing as `Arc::into_raw`.
            let raw = Weak::as_ptr(w);
            WakerArrayInner {
                readiness: Mutex::new(ReadinessArray::new()),
                redirect: [raw; N],
            }
        });

        let wakers =
            array::from_fn(|i| unsafe { waker_from_redirect_position(Arc::clone(&inner), i) });

        Self { inner, wakers }
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Waker> {
        self.wakers.get(index)
    }

    /// Access the `Readiness`.
    pub(crate) fn readiness(&self) -> MutexGuard<'_, ReadinessArray<N>> {
        self.inner.readiness.lock().unwrap()
    }
}

#[deny(unsafe_op_in_unsafe_fn)]
unsafe impl<const N: usize> SharedArcContent for WakerArrayInner<N> {
    fn get_redirect_slice(&self) -> &[*const Self] {
        &self.redirect
    }

    fn wake_index(&self, index: usize) {
        self.readiness.lock().unwrap().wake(index);
    }
}

#[cfg(test)]
mod tests {
    use async_ui_internal_utils::dummy_waker::dummy_waker;

    use super::*;
    #[test]
    fn check_refcount() {
        let mut wa = WakerArray::<5>::new();

        // Each waker holds 1 ref, and the combinator itself holds 1.
        assert_eq!(Arc::strong_count(&wa.inner), 6);

        wa.wakers[4] = dummy_waker();
        assert_eq!(Arc::strong_count(&wa.inner), 5);
        let cloned = wa.wakers[3].clone();
        assert_eq!(Arc::strong_count(&wa.inner), 6);
        wa.wakers[3] = wa.wakers[4].clone();
        assert_eq!(Arc::strong_count(&wa.inner), 5);
        drop(cloned);
        assert_eq!(Arc::strong_count(&wa.inner), 4);

        wa.wakers[0].wake_by_ref();
        wa.wakers[0].wake_by_ref();
        wa.wakers[0].wake_by_ref();
        assert_eq!(Arc::strong_count(&wa.inner), 4);

        wa.wakers[0] = wa.wakers[1].clone();
        assert_eq!(Arc::strong_count(&wa.inner), 4);

        let taken = std::mem::replace(&mut wa.wakers[2], dummy_waker());
        assert_eq!(Arc::strong_count(&wa.inner), 4);
        taken.wake_by_ref();
        assert_eq!(Arc::strong_count(&wa.inner), 4);
        taken.clone().wake();
        assert_eq!(Arc::strong_count(&wa.inner), 4);
        taken.wake();
        assert_eq!(Arc::strong_count(&wa.inner), 3);

        wa.wakers = array::from_fn(|_| dummy_waker());
        assert_eq!(Arc::strong_count(&wa.inner), 1);

        let weak = Arc::downgrade(&wa.inner);
        drop(wa);
        assert_eq!(weak.strong_count(), 0);
    }
}
