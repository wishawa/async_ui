use std::ops::Deref;

use crate::{
    impls::{leaf::LeafPathBuilder, transparent::TransparentDerefMapper},
    path::Path,
    trackable::Trackable,
};

impl<T: Trackable + ?Sized> Trackable for Box<T> {
    type PathBuilder<P: Path<Out = Self>> = BoxPathBuilder<T, P>;

    fn new_path_builder<P: Path<Out = Self>>(parent: P) -> Self::PathBuilder<P> {
        BoxPathBuilder { inner_path: parent }
    }
}

#[derive(x_bow_macros::IntoInnerPath)]
#[into_inner_path(prefix = crate::trackable)]
pub struct BoxPathBuilder<T: ?Sized, P: Path<Out = Box<T>>> {
    inner_path: P,
}

impl<T: ?Sized, P: Path<Out = Box<T>>> Deref for BoxPathBuilder<T, P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.inner_path
    }
}

impl<T: ?Sized, P: Path<Out = Box<T>> + Clone> Clone for BoxPathBuilder<T, P> {
    fn clone(&self) -> Self {
        Self {
            inner_path: self.inner_path.clone(),
        }
    }
}

impl<T: ?Sized, P: Path<Out = Box<T>> + Copy> Copy for BoxPathBuilder<T, P> {}

impl<T: Trackable + ?Sized, P: Path<Out = Box<T>>> BoxPathBuilder<T, P> {
    pub fn content(self) -> T::PathBuilder<TransparentDerefMapper<Box<T>, P>> {
        T::new_path_builder(TransparentDerefMapper::new(self.inner_path))
    }
    pub fn content_shallow(self) -> LeafPathBuilder<TransparentDerefMapper<Box<T>, P>> {
        LeafPathBuilder::new(TransparentDerefMapper::new(self.inner_path))
    }
}
