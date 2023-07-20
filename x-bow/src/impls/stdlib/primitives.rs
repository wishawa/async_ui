use crate::impls::leaf::LeafPathBuilder;
use crate::path::Path;
use crate::trackable::Trackable;

macro_rules! leaf_primitive {
    ($ty:ty) => {
        impl Trackable for $ty {
            type PathBuilder<P: Path<Out = Self>> = LeafPathBuilder<P>;

            fn new_path_builder<P: Path<Out = Self>>(parent: P) -> Self::PathBuilder<P> {
                LeafPathBuilder::new(parent)
            }
        }
    };
}

leaf_primitive!(bool);
leaf_primitive!(char);
leaf_primitive!(f32);
leaf_primitive!(f64);
leaf_primitive!(i128);
leaf_primitive!(i16);
leaf_primitive!(i32);
leaf_primitive!(i64);
leaf_primitive!(i8);
leaf_primitive!(isize);
leaf_primitive!(u128);
leaf_primitive!(u16);
leaf_primitive!(u32);
leaf_primitive!(u64);
leaf_primitive!(u8);
leaf_primitive!(usize);
leaf_primitive!(str);
leaf_primitive!(());

impl<'a> Trackable for &'a str {
    type PathBuilder<P: Path<Out = Self>> = LeafPathBuilder<P>;

    fn new_path_builder<P: Path<Out = Self>>(parent: P) -> Self::PathBuilder<P> {
        LeafPathBuilder::new(parent)
    }
}
