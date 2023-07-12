use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

use crate::{guarantee::PathExtGuaranteed, path::Path};

pub struct TransparentDerefMapper<T, P: Path<Out = T>> {
    parent: P,
}

impl<T, P: Path<Out = T>> TransparentDerefMapper<T, P> {
    pub fn new(parent: P) -> Self {
        Self { parent }
    }
}

impl<T, P: Path<Out = T> + Clone> Clone for TransparentDerefMapper<T, P> {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent.clone(),
        }
    }
}

impl<T: Deref + DerefMut, P: Path<Out = T>> Path for TransparentDerefMapper<T, P> {
    type Out = T::Target;

    fn path_borrow(&self) -> Option<std::cell::Ref<'_, Self::Out>> {
        self.parent.path_borrow().map(|r| Ref::map(r, |t| &**t))
    }

    fn path_borrow_mut(&self) -> Option<std::cell::RefMut<'_, Self::Out>> {
        self.parent
            .path_borrow_mut()
            .map(|r| RefMut::map(r, |t| &mut **t))
    }
    fn visit_hashes(&self, visitor: &mut crate::hash_visitor::HashVisitor) {
        self.parent.visit_hashes(visitor);
        use std::hash::Hasher;
        visitor.write_u8(0);
        visitor.finish_one();
    }
    fn store_wakers(&self) -> &std::cell::RefCell<crate::wakers::StoreWakers> {
        self.parent.store_wakers()
    }
}

impl<T: Deref + DerefMut, P: Path<Out = T> + PathExtGuaranteed> PathExtGuaranteed
    for TransparentDerefMapper<T, P>
{
}
