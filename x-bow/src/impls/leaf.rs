use std::rc::Rc;

use crate::{is_guaranteed::IsGuaranteed, node_down::NodeDownTrait, node_up::NodeUpTrait};
pub struct NodeDownLeaf<'u, T, const G: bool> {
    up: Rc<dyn NodeUpTrait<Data = T> + 'u>,
}

impl<'u, T, const G: bool> Clone for NodeDownLeaf<'u, T, G> {
    fn clone(&self) -> Self {
        Self {
            up: self.up.clone(),
        }
    }
}

impl<'u, T, const G: bool> NodeDownTrait<'u, T> for NodeDownLeaf<'u, T, G> {
    fn invalidate_down(&self) {
        // NO-OP
    }
    fn node_up(&self) -> &Rc<dyn NodeUpTrait<Data = T> + 'u> {
        &self.up
    }
}

impl<'u, T, const G: bool> IsGuaranteed<G> for NodeDownLeaf<'u, T, G> {}

pub trait TrackableLeaf {
    type NodeDown<'u, const G: bool>;
    fn new_node<'u, const G: bool>(
        up_node: Rc<dyn NodeUpTrait<Data = Self> + 'u>,
    ) -> Self::NodeDown<'u, G>;
}

impl<T> TrackableLeaf for T {
    type NodeDown<'u, const G: bool> = NodeDownLeaf<'u, T, G>;
    fn new_node<'u, const G: bool>(
        up_node: Rc<dyn NodeUpTrait<Data = Self> + 'u>,
    ) -> Self::NodeDown<'u, G> {
        NodeDownLeaf { up: up_node }
    }
}
