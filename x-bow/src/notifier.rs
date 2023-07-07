use std::cell::RefCell;

use crate::wakers::StoreWakers;

pub struct Notifier<'b> {
    store: &'b RefCell<StoreWakers>,
    hash: u64,
}

impl<'b> Notifier<'b> {
    pub(crate) fn new(store: &'b RefCell<StoreWakers>, hash: u64) -> Self {
        Self { store, hash }
    }
}

impl<'b> Drop for Notifier<'b> {
    fn drop(&mut self) {
        self.store.borrow_mut().get_entry(self.hash).wake();
    }
}
