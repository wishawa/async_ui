use crate::{
    is_guaranteed::IsGuaranteed, node_down::NodeDownTrait, node_up::NodeUpTrait, shared::Shared,
};
pub struct NodeDownLeaf<'u, T: 'u, const G: bool> {
    up: &'u (dyn NodeUpTrait<Data = T> + 'u),
}

impl<'u, T, const G: bool> NodeDownTrait<'u, T> for NodeDownLeaf<'u, T, G> {
    fn invalidate_downward(&self) {
        // no-op
    }
    fn node_up(&self) -> &'u (dyn NodeUpTrait<Data = T> + 'u) {
        self.up
    }
}

impl<'u, T, const G: bool> IsGuaranteed<G> for NodeDownLeaf<'u, T, G> {}

pub trait TrackableLeaf {
    type NodeDown<'u, const G: bool>
    where
        Self: 'u;
    fn new_node<'u, Up: NodeUpTrait<Data = Self> + 'u, const G: bool>(
        shared: &'u Shared,
        up_node: &'u Up,
    ) -> Self::NodeDown<'u, G>;
}

impl<T> TrackableLeaf for T {
    type NodeDown<'u, const G: bool> = NodeDownLeaf<'u, T, G> where T: 'u;
    fn new_node<'u, Up: NodeUpTrait<Data = Self> + 'u, const G: bool>(
        _shared: &'u Shared,
        up_node: &'u Up,
    ) -> Self::NodeDown<'u, G> {
        NodeDownLeaf { up: up_node }
    }
}
