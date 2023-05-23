use core::task::Waker;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use super::{
    super::shared_arc::{waker_from_redirect_position, SharedArcContent},
    ReadinessVec,
};

/// A collection of wakers sharing the same allocation.
pub(crate) struct WakerVec {
    wakers: Vec<Waker>,
    inner: Arc<WakerVecInner>,
}

/// See [super::super::shared_arc] for how this works.
struct WakerVecInner {
    redirect: Vec<*const Self>,
    readiness: Mutex<ReadinessVec>,
}

impl WakerVec {
    /// Create a new instance of `WakerVec`.
    pub(crate) fn new(len: usize) -> Self {
        let inner = Arc::new_cyclic(|w| {
            // `Weak::as_ptr` on a live Weak gives the same thing as `Arc::into_raw`.
            let raw = Weak::as_ptr(w);
            WakerVecInner {
                readiness: Mutex::new(ReadinessVec::new(len)),
                redirect: vec![raw; len],
            }
        });

        // Now the redirect vec is complete. Time to create the actual Wakers.
        let wakers = (0..len)
            .map(|i| unsafe { waker_from_redirect_position(Arc::clone(&inner), i) })
            .collect();

        Self { inner, wakers }
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Waker> {
        self.wakers.get(index)
    }

    pub(crate) fn readiness(&self) -> MutexGuard<'_, ReadinessVec> {
        self.inner.readiness.lock().unwrap()
    }
}

#[deny(unsafe_op_in_unsafe_fn)]
unsafe impl SharedArcContent for WakerVecInner {
    fn get_redirect_slice(&self) -> &[*const Self] {
        &self.redirect
    }

    fn wake_index(&self, index: usize) {
        self.readiness.lock().unwrap().wake(index);
    }
}
