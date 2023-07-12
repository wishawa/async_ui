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

/// A trait for handling paths generically.
///
/// An `impl Tracked<T>` type gives you a path that points to `T`.
/// You can use the [PathExt][crate::PathExt] methods on `T`.
///
/// You can also continue the path into substructures of `T` by using
/// `build_path()` to obtain the [PathBuilder][crate::Trackable::PathBuilder].
///
/// It is recommended to avoid using this trait. If you can, you should instead
/// pass around the [StoreRoot][crate::StoreRoot] and build your paths from
/// there every time.
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
