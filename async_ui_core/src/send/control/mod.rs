use self::{node_guard::NodeGuard, position::PositionIndices, vnode::VNode};

use super::backend::Backend;
pub mod node_guard;
pub mod position;
pub mod vnode;

#[derive(Debug)]
pub struct Control<B: Backend> {
    vnode: B::VNode,
    position: PositionIndices,
}

impl<B: Backend> Clone for Control<B> {
    fn clone(&self) -> Self {
        Self {
            vnode: self.vnode.clone(),
            position: self.position.clone(),
        }
    }
}

impl<B: Backend> Control<B> {
    pub fn nest(&self, index: usize) -> Self {
        let mut new = self.clone();
        new.position.nest(index);
        new
    }
    pub fn put_node(&self, node: <B::VNode as VNode>::Node) -> NodeGuard<B> {
        self.vnode.ins_node(self.position.clone(), node);
        NodeGuard::new(self.vnode.clone(), self.position.clone())
    }
    pub fn new_with_vnode(vnode: B::VNode) -> Self {
        Self {
            vnode,
            position: PositionIndices::default(),
        }
    }
    pub fn get_vnode(&self) -> &B::VNode {
        &self.vnode
    }
    pub fn get_position(&self) -> &PositionIndices {
        &self.position
    }
}
