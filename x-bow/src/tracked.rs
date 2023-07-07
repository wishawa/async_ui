use std::ops::Deref;

use crate::{Path, Trackable};

pub trait DerefToPath<T: ?Sized>: Deref<Target = Self::Path> {
    type Path: Path<Out = T> + Clone;
}

impl<T: ?Sized> DerefToPath<<T::Target as Path>::Out> for T
where
    T: Deref,
    T::Target: Path + Clone + Sized,
{
    type Path = T::Target;
}

pub trait Tracked<T: Trackable + ?Sized>: DerefToPath<T> + Clone {
    fn build_path(self) -> T::PathBuilder<Self::Target>;
}

impl<B> Tracked<<B::Target as Path>::Out> for B
where
    B: Deref + Clone,
    B::Target: Path + Clone + Sized,
    <B::Target as Path>::Out: Trackable,
    B: Into<<<B::Target as Path>::Out as Trackable>::PathBuilder<B::Target>>,
{
    fn build_path(self) -> <<B::Target as Path>::Out as Trackable>::PathBuilder<Self::Target> {
        self.into()
    }
}
