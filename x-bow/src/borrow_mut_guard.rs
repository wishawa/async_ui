use std::{
    cell::{RefCell, RefMut},
    ops::{Deref, DerefMut},
};

use crate::{notifier::Notifier, wakers::StoreWakers};

pub struct BorrowMutGuard<'b, U: ?Sized> {
    inner: RefMut<'b, U>,
    _notifier: Notifier<'b>,
}

impl<'b, U: ?Sized> BorrowMutGuard<'b, U> {
    pub(crate) fn new(inner: RefMut<'b, U>, store: &'b RefCell<StoreWakers>, hash: u64) -> Self {
        Self {
            inner,
            _notifier: Notifier::new(store, hash),
        }
    }
}

impl<'b, U: ?Sized> Deref for BorrowMutGuard<'b, U> {
    type Target = U;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
impl<'b, U: ?Sized> DerefMut for BorrowMutGuard<'b, U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}
