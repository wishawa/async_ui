use std::{
    cell::{Cell, RefCell},
    task::Waker,
};

pub struct Listeners {
    inner: RefCell<ListenersInner>,
    inside_version: Cell<u64>,
    outside_version: Cell<u64>,
}
struct ListenersInner {
    outside_wakers: Vec<Waker>,
    inside_wakers: Vec<Waker>,
}

impl Listeners {
    pub const fn new() -> Self {
        let inner = RefCell::new(ListenersInner {
            outside_wakers: Vec::new(),
            inside_wakers: Vec::new(),
        });
        Self {
            inner,
            inside_version: Cell::new(0),
            outside_version: Cell::new(0),
        }
    }
    pub(crate) fn invalidate_inside(&self) {
        self.inside_version.set(self.inside_version.get() + 1);
        self.inner
            .borrow_mut()
            .inside_wakers
            .drain(..)
            .for_each(Waker::wake);
    }
    pub(crate) fn invalidate_outside(&self) {
        self.outside_version.set(self.outside_version.get() + 1);
        self.inner
            .borrow_mut()
            .outside_wakers
            .drain(..)
            .for_each(Waker::wake);
    }
    pub(crate) fn add_inside_waker(&self, waker: Waker) {
        self.inner.borrow_mut().inside_wakers.push(waker)
    }
    pub(crate) fn add_outside_waker(&self, waker: Waker) {
        self.inner.borrow_mut().outside_wakers.push(waker)
    }
    pub(crate) fn inside_version(&self) -> u64 {
        self.inside_version.get()
    }
    pub(crate) fn outside_version(&self) -> u64 {
        self.outside_version.get()
    }
}
