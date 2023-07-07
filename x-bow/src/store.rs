use std::cell::{Ref, RefCell};

use crate::{guarantee::PathExtGuaranteed, path::Path, trackable::Trackable, wakers::StoreWakers};

pub struct Store<S> {
    data: RefCell<S>,
    wakers: RefCell<StoreWakers>,
}

impl<S> Store<S> {
    /// Create a new store with the given data.
    /// This puts the data in a [RefCell] and set up all the change listening
    /// mechanisms.
    pub fn new(data: S) -> Self {
        Self {
            data: RefCell::new(data),
            wakers: RefCell::new(StoreWakers::new()),
        }
    }
}
impl<S: Trackable> Store<S> {
    pub fn build_path<'s>(&'s self) -> StoreRoot<'s, S> {
        S::new_path_builder(RootPath { store: self })
    }
}
pub type StoreRoot<'s, S> = <S as Trackable>::PathBuilder<RootPath<'s, S>>;

pub struct RootPath<'s, S> {
    store: &'s Store<S>,
}

impl<'s, S> Clone for RootPath<'s, S> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
        }
    }
}
impl<'s, S> Copy for RootPath<'s, S> {}

impl<'s, S> Path for RootPath<'s, S> {
    type Out = S;

    fn path_borrow<'d>(&'d self) -> Option<Ref<'d, Self::Out>>
    where
        Self: 'd,
    {
        Some(self.store.data.borrow())
    }

    fn path_borrow_mut<'d>(&'d self) -> Option<std::cell::RefMut<'d, Self::Out>>
    where
        Self: 'd,
    {
        Some(self.store.data.borrow_mut())
    }

    fn visit_hashes(&self, visitor: &mut crate::hash_visitor::HashVisitor) {
        visitor.finish_one();
    }

    fn store_wakers(&self) -> &RefCell<StoreWakers> {
        &self.store.wakers
    }
}
impl<'s, S> PathExtGuaranteed for RootPath<'s, S> {}
