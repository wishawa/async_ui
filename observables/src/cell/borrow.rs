use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
    task::Waker,
};

use super::ObserbableCellInner;

pub struct ObservableCellBorrow<'b, T> {
    pub(super) reference: Ref<'b, ObserbableCellInner<T>>,
}

impl<'b, T> Deref for ObservableCellBorrow<'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.reference.data
    }
}
pub struct ObservableCellBorrowMut<'b, T> {
    pub(super) reference: RefMut<'b, ObserbableCellInner<T>>,
}

impl<'b, T> DerefMut for ObservableCellBorrowMut<'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reference.data
    }
}

impl<'b, T> Deref for ObservableCellBorrowMut<'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.reference.data
    }
}

impl<'b, T> Drop for ObservableCellBorrowMut<'b, T> {
    fn drop(&mut self) {
        self.reference.version = self.reference.version.incremented();
        self.reference.listeners.drain(..).for_each(Waker::wake);
    }
}
