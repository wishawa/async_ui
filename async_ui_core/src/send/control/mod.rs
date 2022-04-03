use self::{node_guard::NodeGuard, position::PositionIndices, vnode::VNode};

use super::{backend::Backend, Shared};
pub mod node_guard;
pub mod position;
pub mod vnode;

pub type VNodeWrap<B> = Shared<dyn VNode<B>>;

#[derive(Debug)]
pub struct Control<B: Backend> {
    vnode: Shared<dyn VNode<B>>,
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
    pub fn put_node(&self, node: B::NodeType) -> NodeGuard<B> {
        self.vnode.ins_node(self.position.clone(), node);
        NodeGuard::new(self.vnode.clone(), self.position.clone())
    }
    pub fn new_with_vnode(vnode: VNodeWrap<B>) -> Self {
        Self {
            vnode,
            position: PositionIndices::default(),
        }
    }
    pub fn get_vnode(&self) -> &VNodeWrap<B> {
        &self.vnode
    }
    pub fn get_position(&self) -> &PositionIndices {
        &self.position
    }
}
