use std::rc::Rc;

use crate::{is_guaranteed::IsGuaranteed, node_down::NodeDownTrait, node_up::NodeUpTrait};

pub trait Trackable {
    type NodeDown<'u, const G: bool>: NodeDownTrait<'u, Self> + IsGuaranteed<G> + Clone
    where
        Self: 'u;
    #[doc(hidden)]
    fn new_node<'u, const G: bool>(
        up_node: Rc<dyn NodeUpTrait<Data = Self> + 'u>,
    ) -> Self::NodeDown<'u, G>
    where
        Self: 'u;
}
