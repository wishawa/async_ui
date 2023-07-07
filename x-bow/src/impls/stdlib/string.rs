use std::ops::Deref;

use crate::{impls::transparent::TransparentDerefMapper, path::Path, trackable::Trackable};

impl Trackable for String {
    type PathBuilder<P: Path<Out = Self>> = StringPathBuilder<P>;

    fn new_path_builder<P: Path<Out = Self>>(parent: P) -> Self::PathBuilder<P> {
        StringPathBuilder { inner_path: parent }
    }
}

#[derive(Clone, Copy, x_bow_macros::IntoInnerPath)]
#[into_inner_path(prefix = crate::trackable)]
pub struct StringPathBuilder<P: Path<Out = String>> {
    inner_path: P,
}

impl<P: Path<Out = String>> Deref for StringPathBuilder<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.inner_path
    }
}

impl<P: Path<Out = String>> StringPathBuilder<P> {
    pub fn content(self) -> <str as Trackable>::PathBuilder<TransparentDerefMapper<String, P>> {
        str::new_path_builder(TransparentDerefMapper::new(self.inner_path))
    }
}
