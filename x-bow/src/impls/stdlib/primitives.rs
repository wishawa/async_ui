use crate::impls::leaf::{NodeDownLeaf, TrackableLeaf};
use crate::node_up::NodeUpTrait;
use crate::shared::Shared;
use crate::trackable::Trackable;

macro_rules! leaf_primitive {
    ($primitive:ty) => {
        impl Trackable for $primitive {
            type NodeDown<'u, const G: bool> = NodeDownLeaf<'u, Self, G> where Self: 'u;
            fn new_node<'u, Up: NodeUpTrait<Data = Self> + 'u, const G: bool>(
                shared: &'u Shared,
                up_node: &'u Up,
            ) -> Self::NodeDown<'u, G>
            where
                Self: 'u,
            {
                <Self as TrackableLeaf>::new_node(shared, up_node)
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
leaf_primitive!(String);
leaf_primitive!(());
