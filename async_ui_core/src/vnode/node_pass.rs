use std::rc::Rc;

use crate::{backend::BackendTrait, context::ContextMap, position::PositionIndex, vnode::VNode};

use super::VNodeTrait;

pub struct PassVNode<B: BackendTrait> {
    parent: Rc<VNode<B>>,
    index: usize,
    context: ContextMap,
}

impl<B: BackendTrait> PassVNode<B> {
    pub fn new(parent: Rc<VNode<B>>, index: usize) -> Self {
        let context = parent.get_context_map().to_owned();
        Self {
            parent,
            index,
            context,
        }
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

    fn get_context_map<'s>(&'s self) -> &'s ContextMap {
        &self.context
    }
}
