use std::ops::Deref;

use crate::{Path, PathExtGuaranteed, Trackable};

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
pub trait Tracked<T: Trackable + ?Sized>: Deref<Target = Self::Path> + Clone {
    type Path: Path<Out = T> + Clone;
    fn build_path(self) -> T::PathBuilder<Self::Target>;
}

impl<B> Tracked<<B::Target as Path>::Out> for B
where
    B: Deref + Clone,
    B::Target: Path + Clone + Sized,
    <B::Target as Path>::Out: Trackable,
    B: Into<<<B::Target as Path>::Out as Trackable>::PathBuilder<B::Target>>,
{
    type Path = B::Target;
    fn build_path(self) -> <<B::Target as Path>::Out as Trackable>::PathBuilder<Self::Target> {
        self.into()
    }
}

/// Like [Tracked], but for paths known to be [guaranteed][crate::PathExtGuaranteed].
pub trait TrackedGuaranteed<T: Trackable + ?Sized>:
    Tracked<T, Path = Self::PathGuaranteed> + Deref<Target = Self::PathGuaranteed>
{
    type PathGuaranteed: PathExtGuaranteed<Out = T>;
}

impl<B> TrackedGuaranteed<<B::Target as Path>::Out> for B
where
    B: Deref + Clone,
    B::Target: PathExtGuaranteed + Clone + Sized,
    <B::Target as Path>::Out: Trackable,
    B: Into<<<B::Target as Path>::Out as Trackable>::PathBuilder<B::Target>>,
{
    type PathGuaranteed = B::Target;
}
