use std::{
    cell::RefMut,
    ops::{Deref, DerefMut},
    task::Waker,
};

use super::Inner;

pub struct ReactiveCellBorrowMut<'b, T> {
    pub(super) reference: RefMut<'b, Inner<T>>,
}

impl<'b, T> DerefMut for ReactiveCellBorrowMut<'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reference.data
    }
}

impl<'b, T> Deref for ReactiveCellBorrowMut<'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.reference.data
    }
}

impl<'b, T> Drop for ReactiveCellBorrowMut<'b, T> {
    fn drop(&mut self) {
        self.reference.version += 1;
        self.reference.listeners.drain(..).for_each(Waker::wake);
    }
}
