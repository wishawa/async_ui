use std::{cell::RefCell, task::Waker};

pub struct Listeners {
    inner: RefCell<ListenersInner>,
}
struct ListenersInner {
    wakers: Vec<Waker>,
    version: u64,
}

impl Listeners {
    pub const fn new() -> Self {
        let inner = RefCell::new(ListenersInner {
            wakers: Vec::new(),
            version: 0,
        });
        Self { inner }
    }
    pub(crate) fn fire(&self) {
        let mut bm = self.inner.borrow_mut();
        bm.version += 1;
        bm.wakers.iter().for_each(Waker::wake_by_ref);
    }
}
