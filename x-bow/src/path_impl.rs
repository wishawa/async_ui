use crate::{hash_visitor::HashVisitor, path::Path, wakers::StoreWakers, PathExtGuaranteed};
use std::cell::{Ref, RefCell, RefMut};

impl<'t, T> Path for std::rc::Rc<dyn Path<Out = T> + 't> {
    type Out = T;

    fn path_borrow(&self) -> Option<Ref<'_, Self::Out>> {
        (**self).path_borrow()
    }

    fn path_borrow_mut(&self) -> Option<RefMut<'_, Self::Out>> {
        (**self).path_borrow_mut()
    }

    fn visit_hashes(&self, visitor: &mut HashVisitor) {
        (**self).visit_hashes(visitor)
    }

    fn store_wakers(&self) -> &RefCell<StoreWakers> {
        (**self).store_wakers()
    }
}

impl<'t, T> Path for Box<dyn Path<Out = T> + 't> {
    type Out = T;

    fn path_borrow(&self) -> Option<Ref<'_, Self::Out>> {
        (**self).path_borrow()
    }

    fn path_borrow_mut(&self) -> Option<RefMut<'_, Self::Out>> {
        (**self).path_borrow_mut()
    }

    fn visit_hashes(&self, visitor: &mut HashVisitor) {
        (**self).visit_hashes(visitor)
    }

    fn store_wakers(&self) -> &RefCell<StoreWakers> {
        (**self).store_wakers()
    }
}

pub struct ReferencePath<'r, P: Path + ?Sized>(&'r P);

impl<'r, P: Path + ?Sized> ReferencePath<'r, P> {
    pub(crate) fn new(path: &'r P) -> Self {
        Self(path)
    }
}

impl<'r, P: Path + ?Sized> Clone for ReferencePath<'r, P> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<'r, P: Path + ?Sized> Copy for ReferencePath<'r, P> {}

impl<'r, P: Path + ?Sized> Path for ReferencePath<'r, P> {
    type Out = P::Out;

    fn path_borrow(&self) -> Option<Ref<'_, Self::Out>> {
        self.0.path_borrow()
    }
    fn path_borrow_mut(&self) -> Option<RefMut<'_, Self::Out>> {
        self.0.path_borrow_mut()
    }
    fn visit_hashes(&self, visitor: &mut HashVisitor) {
        self.0.visit_hashes(visitor)
    }
    fn store_wakers(&self) -> &RefCell<StoreWakers> {
        self.0.store_wakers()
    }
}

impl<'r, P: PathExtGuaranteed + ?Sized> PathExtGuaranteed for ReferencePath<'r, P> {}
