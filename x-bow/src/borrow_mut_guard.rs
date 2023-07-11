use std::{
    cell::{RefCell, RefMut},
    ops::{Deref, DerefMut},
};

use crate::{notifier::Notifier, wakers::StoreWakers, Path};

pub struct BorrowMutGuard<'b, P: Path + ?Sized> {
    inner: RefMut<'b, P::Out>,
    _notifier: Notifier<'b, P>,
}

impl<'b, P: Path + ?Sized> BorrowMutGuard<'b, P> {
    pub(crate) fn new(
        inner: RefMut<'b, P::Out>,
        store: &'b RefCell<StoreWakers>,
        path: &'b P,
    ) -> Self {
        Self {
            inner,
            _notifier: Notifier::new(store, path),
        }
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
