use std::{
    cell::{Ref, RefMut},
    hash::Hasher,
    ops::Deref,
};

use crate::{impls::leaf::LeafPathBuilder, path::Path, trackable::Trackable};

#[derive(x_bow_macros::IntoInnerPath)]
#[into_inner_path(prefix = crate::trackable)]
pub struct VecPathBuilder<T, P: Path<Out = Vec<T>>> {
    inner_path: P,
}

impl<T, P: Path<Out = Vec<T>>> Deref for VecPathBuilder<T, P> {
    type Target = P;
    fn deref(&self) -> &Self::Target {
        &self.inner_path
    }
}

impl<T> Trackable for Vec<T> {
    type PathBuilder<P: Path<Out = Self>> = VecPathBuilder<T, P>;

    fn new_path_builder<P: Path<Out = Self>>(parent: P) -> Self::PathBuilder<P> {
        VecPathBuilder { inner_path: parent }
    }
}

impl<T, P: Path<Out = Vec<T>> + Clone> Clone for VecPathBuilder<T, P> {
    fn clone(&self) -> Self {
        Self {
            inner_path: self.inner_path.clone(),
        }
    }
}
impl<T, P: Path<Out = Vec<T>> + Copy> Copy for VecPathBuilder<T, P> {}

impl<T: Trackable, P: Path<Out = Vec<T>>> VecPathBuilder<T, P> {
    pub fn index(self, index: usize) -> T::PathBuilder<VecIndexMapper<T, P>> {
        T::new_path_builder(VecIndexMapper {
            parent: self.inner_path,
            index,
        })
    }
}

impl<T, P: Path<Out = Vec<T>>> VecPathBuilder<T, P> {
    pub fn index_shallow(self, index: usize) -> LeafPathBuilder<VecIndexMapper<T, P>> {
        LeafPathBuilder::new(VecIndexMapper {
            parent: self.inner_path,
            index,
        })
    }
}

pub struct VecIndexMapper<T, P: Path<Out = Vec<T>>> {
    parent: P,
    index: usize,
}

impl<T, P: Path<Out = Vec<T>> + Clone> Clone for VecIndexMapper<T, P> {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent.clone(),
            index: self.index.clone(),
        }
    }
}
impl<T, P: Path<Out = Vec<T>> + Copy> Copy for VecIndexMapper<T, P> {}

impl<T, P: Path<Out = Vec<T>>> Path for VecIndexMapper<T, P> {
    type Out = T;

    fn path_borrow<'d>(&'d self) -> Option<std::cell::Ref<'d, Self::Out>>
    where
        Self: 'd,
    {
        self.parent
            .path_borrow()
            .and_then(|r| Ref::filter_map(r, |s| s.get(self.index)).ok())
    }
    fn path_borrow_mut<'d>(&'d self) -> Option<std::cell::RefMut<'d, Self::Out>>
    where
        Self: 'd,
    {
        self.parent
            .path_borrow_mut()
            .and_then(|r| RefMut::filter_map(r, |s| s.get_mut(self.index)).ok())
    }

    fn visit_hashes(&self, visitor: &mut crate::hash_visitor::HashVisitor) {
        self.parent.visit_hashes(visitor);
        visitor.write_usize(self.index);
        visitor.finish_one();
    }
    fn store_wakers(&self) -> &std::cell::RefCell<crate::wakers::StoreWakers> {
        self.parent.store_wakers()
    }
}
