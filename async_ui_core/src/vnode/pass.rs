use std::rc::Rc;

use crate::{backend::BackendTrait, position::PositionIndex, vnode::VNode};

use super::VNodeTrait;

pub struct PassVNode<B: BackendTrait> {
    parent: Rc<VNode<B>>,
    index: usize,
}

impl<B: BackendTrait> PassVNode<B> {
    pub fn new(parent: Rc<VNode<B>>, index: usize) -> Self {
        Self { parent, index }
    }
}

impl<B: BackendTrait> VNodeTrait<B> for PassVNode<B> {
    fn add_child_node(&self, node: B::Node, mut position: PositionIndex) {
        position.wrap(self.index);
        self.parent.add_child_node(node, position)
    }

    fn del_child_node(&self, mut position: PositionIndex) {
        position.wrap(self.index);
        self.parent.del_child_node(position)
    }
}
