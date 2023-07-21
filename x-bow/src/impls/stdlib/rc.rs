use std::{
    cell::{Ref, RefMut},
    fmt::Debug,
    hash::Hasher,
    ops::Deref,
    rc::Rc,
};

use crate::{path::Path, trackable::Trackable};

impl<T: Trackable> Trackable for Rc<T> {
    type PathBuilder<P: Path<Out = Self>> = RcPathBuilder<T, P>;

    fn new_path_builder<P: Path<Out = Self>>(parent: P) -> Self::PathBuilder<P> {
        RcPathBuilder { inner_path: parent }
    }
}

#[derive(x_bow_macros::IntoPath)]
#[into_path(prefix = crate::trackable)]
pub struct RcPathBuilder<T, P: Path<Out = Rc<T>>> {
    inner_path: P,
}

impl<T, P: Path<Out = Rc<T>>> Deref for RcPathBuilder<T, P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.inner_path
    }
}

impl<T: Trackable, P: Path<Out = Rc<T>>> RcPathBuilder<T, P> {
    pub fn content(self) -> T::PathBuilder<RcMapper<T, P>> {
        T::new_path_builder(RcMapper {
            parent: self.inner_path,
        })
    }
}

pub struct RcMapper<T: ?Sized, P: Path<Out = Rc<T>>> {
    parent: P,
}

impl<T: ?Sized, P: Path<Out = Rc<T>> + Clone> Clone for RcMapper<T, P> {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent.clone(),
        }
    }
}
impl<T: ?Sized, P: Path<Out = Rc<T>> + Copy> Copy for RcMapper<T, P> {}

impl<T: ?Sized, P: Path<Out = Rc<T>> + Debug> Debug for RcMapper<T, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.parent.fmt(f)?;
        f.write_str("→(Rc content)")
    }
}
impl<T: ?Sized, P: Path<Out = Rc<T>>> Path for RcMapper<T, P> {
    type Out = T;

    fn path_borrow(&self) -> Option<std::cell::Ref<'_, Self::Out>> {
        self.parent.path_borrow().map(|r| Ref::map(r, |rc| &**rc))
    }
    fn path_borrow_mut(&self) -> Option<std::cell::RefMut<'_, Self::Out>> {
        self.parent
            .path_borrow_mut()
            .and_then(|r| RefMut::filter_map(r, |rc| Rc::get_mut(rc)).ok())
    }

    fn visit_hashes(&self, visitor: &mut crate::hash_visitor::HashVisitor) {
        self.parent.visit_hashes(visitor);
        visitor.write_u8(0);
        visitor.finish_one();
    }
    fn store_wakers(&self) -> &std::cell::RefCell<crate::wakers::StoreWakers> {
        self.parent.store_wakers()
    }
}
