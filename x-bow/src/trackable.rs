use std::ops::Deref;

use crate::path::Path;

/// Allow building [Path]s into parts of this data type.
///
/// Use `#[derive(Trackable)]` on structs or enums to implement this.
pub trait Trackable {
    /// A `PathBuilder` object contains in itself a [Path] pointing to `T`.
    /// It has methods that let you "extend" that path to point to a smaller part
    /// of `T`: a field (if `T` is a struct), an entry (if `T` is a HashMap), etc.
    type PathBuilder<P: Path<Out = Self>>: Deref<Target = P> + IntoPath<IntoPath = P>;

    #[doc(hidden)]
    fn new_path_builder<P: Path<Out = Self>>(parent: P) -> Self::PathBuilder<P>;
}

/// For converting a [PathBuilder][Trackable::PathBuilder] into the [Path] inside it.
pub trait IntoPath {
    type IntoPath;
    /// Get the Path inside the PathBuilder.
    fn into_path(self) -> Self::IntoPath;
}
