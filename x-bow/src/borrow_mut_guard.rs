use std::{
    cell::{RefCell, RefMut},
    hash::Hasher,
    ops::{Deref, DerefMut},
};

use crate::{
    hash::WakerHashEntry,
    hash_visitor::{HashVisitor, HashVisitorBehavior, HasherType},
    wakers::StoreWakers,
    Path,
};

/// A guard similar to [RefMut]. Notifies all the relavant listeners when dropped.
///
/// Obtain this guard through the [borrow_mut][crate::PathExtGuaranteed::borrow_mut]
/// or [borrow_opt_mut][crate::PathExt::borrow_opt_mut] method.
pub struct BorrowMutGuard<'b, P: Path + ?Sized> {
    inner: RefMut<'b, P::Out>,
    store: &'b RefCell<StoreWakers>,
    path: &'b P,
}

impl<'b, P: Path + ?Sized> BorrowMutGuard<'b, P> {
    pub(crate) fn new(
        inner: RefMut<'b, P::Out>,
        store: &'b RefCell<StoreWakers>,
        path: &'b P,
    ) -> Self {
        Self { inner, store, path }
    }
}

impl<'b, P: Path + ?Sized> Deref for BorrowMutGuard<'b, P> {
    type Target = P::Out;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
impl<'b, P: Path + ?Sized> DerefMut for BorrowMutGuard<'b, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

impl<'b, P: Path + ?Sized> Drop for BorrowMutGuard<'b, P> {
    fn drop(&mut self) {
        notify(self.store, self.path);
    }
}

fn notify<P: Path + ?Sized>(store: &RefCell<StoreWakers>, path: &P) {
    let wakers = &mut *store.borrow_mut();
    let mut visitor = HashVisitor {
        hasher: HasherType::new(),
        behavior: HashVisitorBehavior::WakeListeners { wakers },
    };
    path.visit_hashes(&mut visitor);
    let hash = WakerHashEntry::regular_from(visitor.hasher.finish());
    wakers.get_entry(hash).wake();
}
