use crate::{hash_visitor::HashVisitor, path::Path, wakers::StoreWakers};
use std::cell::{Ref, RefCell, RefMut};

impl<'t, T> Path for std::rc::Rc<dyn Path<Out = T> + 't> {
    type Out = T;

    fn path_borrow<'d>(&'d self) -> Option<Ref<'d, Self::Out>>
    where
        Self: 'd,
    {
        (**self).path_borrow()
    }

    fn path_borrow_mut<'d>(&'d self) -> Option<RefMut<'d, Self::Out>>
    where
        Self: 'd,
    {
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

    fn path_borrow<'d>(&'d self) -> Option<Ref<'d, Self::Out>>
    where
        Self: 'd,
    {
        (**self).path_borrow()
    }

    fn path_borrow_mut<'d>(&'d self) -> Option<RefMut<'d, Self::Out>>
    where
        Self: 'd,
    {
        (**self).path_borrow_mut()
    }

    fn visit_hashes(&self, visitor: &mut HashVisitor) {
        (**self).visit_hashes(visitor)
    }

    fn store_wakers(&self) -> &RefCell<StoreWakers> {
        (**self).store_wakers()
    }
}
