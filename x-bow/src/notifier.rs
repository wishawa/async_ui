use std::{cell::RefCell, hash::Hasher};

use crate::{
    hash::WakerHashEntry,
    hash_visitor::{HashVisitor, HashVisitorBehavior, HasherType},
    wakers::StoreWakers,
    Path,
};

pub struct Notifier<'b, P: Path + ?Sized> {
    store: &'b RefCell<StoreWakers>,
    path: &'b P,
}

impl<'b, P: Path + ?Sized> Notifier<'b, P> {
    pub(crate) fn new(store: &'b RefCell<StoreWakers>, path: &'b P) -> Self {
        Self { store, path }
    }
}

impl<'b, P: Path + ?Sized> Drop for Notifier<'b, P> {
    fn drop(&mut self) {
        let wakers = &mut *self.store.borrow_mut();
        let mut visitor = HashVisitor {
            hasher: HasherType::new(),
            behavior: HashVisitorBehavior::WakeListeners { wakers },
        };
        self.path.visit_hashes(&mut visitor);
        let hash = WakerHashEntry::regular_from(visitor.hasher.finish());
        wakers.get_entry(hash).wake();
    }
}
