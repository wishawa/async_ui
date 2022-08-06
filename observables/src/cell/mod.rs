use std::{cell::RefCell, task::Waker};

use crate::{Mutatable, Observable, ObservableBase};

pub struct ObservableCell<T> {
    inner: RefCell<ObserbableCellInner<T>>,
}

struct ObserbableCellInner<T> {
    data: T,
    listeners: Vec<Waker>,
    version: u64,
}

impl<T> ObservableCell<T> {
    pub fn new(data: T) -> Self {
        let inner = RefCell::new(ObserbableCellInner {
            data,
            listeners: Vec::new(),
            version: 0,
        });
        Self { inner }
    }
}
impl<T> ObservableBase for ObservableCell<T> {
    fn add_waker(&self, waker: Waker) {
        self.inner.borrow_mut().listeners.push(waker);
    }
    fn get_version(&self) -> u64 {
        self.inner.borrow().version
    }
}
impl<T> Observable<T> for ObservableCell<T> {
    fn visit<R, F: FnOnce(&T) -> R>(&self, func: F) -> R {
        let b = self.inner.borrow();
        func(&b.data)
    }
}
impl<T> Mutatable<T> for ObservableCell<T> {
    fn visit_mut<R, F: FnOnce(&mut T) -> R>(&self, func: F) -> R {
        let mut bm = self.inner.borrow_mut();
        func(&mut bm.data)
    }
}
