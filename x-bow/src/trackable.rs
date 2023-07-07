use crate::path::Path;

pub trait Trackable {
    type PathBuilder<P: Path<Out = Self>>: IntoInnerPath<P>;
    #[doc(hidden)]
    fn new_path_builder<P: Path<Out = Self>>(parent: P) -> Self::PathBuilder<P>;
}

pub trait IntoInnerPath<P> {
    fn into_inner_path(self) -> P;
}
