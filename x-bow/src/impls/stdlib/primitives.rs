use crate::impls::leaf::{NodeDownLeaf, TrackableLeaf};
use crate::node_up::NodeUpTrait;
use crate::trackable::Trackable;
use std::rc::Rc;

macro_rules! leaf_primitive {
    ($primitive:ty) => {
        impl Trackable for $primitive {
            type NodeDown<'u, const G: bool> = NodeDownLeaf<'u, Self, G> where Self: 'u;
            fn new_node<'u, const G: bool>(
                up_node: Rc<dyn NodeUpTrait<Data = Self> + 'u>,
            ) -> Self::NodeDown<'u, G>
            where
                Self: 'u,
            {
                <Self as TrackableLeaf>::new_node(up_node)
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
