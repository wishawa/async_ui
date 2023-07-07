use std::ops::Deref;

use crate::{path::Path, trackable::IntoInnerPath};

pub trait TrackableLeaf {
    type PathBuilder<P: Path<Out = Self>>: IntoInnerPath<P>;
    fn new_path_builder<P: Path<Out = Self>>(parent: P) -> Self::PathBuilder<P>;
}
impl<T> TrackableLeaf for T {
    type PathBuilder<P: Path<Out = Self>> = LeafPathBuilder<P>;

    fn new_path_builder<P: Path<Out = Self>>(parent: P) -> Self::PathBuilder<P> {
        LeafPathBuilder::new(parent)
    }
}

#[derive(Clone, Copy, x_bow_macros::IntoInnerPath)]
#[into_inner_path(prefix = crate::trackable)]
pub struct LeafPathBuilder<P: Path> {
    inner_path: P,
}

impl<P: Path> LeafPathBuilder<P> {
    pub fn new(parent: P) -> Self {
        Self { inner_path: parent }
    }
}

impl<P: Path> Deref for LeafPathBuilder<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.inner_path
    }
}
