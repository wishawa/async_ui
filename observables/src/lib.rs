use std::{cell::RefCell, task::Waker};

use slab::Slab;

mod borrow;
mod stream;

pub struct Observable<T> {
    inner: RefCell<ObserbableInner<T>>,
}
struct ObserbableInner<T> {
    data: T,
    listeners: Slab<Waker>,
    version: usize,
}

impl<T> Observable<T> {
    pub fn new(data: T) -> Self {
        let inner = RefCell::new(ObserbableInner {
            data,
            listeners: Slab::new(),
            version: 0,
        });
        Self { inner }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
