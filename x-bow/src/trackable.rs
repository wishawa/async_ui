use crate::{
    is_guaranteed::IsGuaranteed, node_down::NodeDownTrait, node_up::NodeUpTrait, shared::Shared,
};

pub trait Trackable {
    type NodeDown<'u, const G: bool>: NodeDownTrait<'u, Self> + IsGuaranteed<G>
    where
        Self: 'u;
    #[doc(hidden)]
    fn new_node<'u, Up: NodeUpTrait<Data = Self> + 'u, const G: bool>(
        shared: &'u Shared,
        up_node: &'u Up,
    ) -> Self::NodeDown<'u, G>
    where
        Self: 'u;
}
