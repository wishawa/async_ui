use super::super::backend::Backend;

use super::{position::PositionIndices, vnode::VNode};

pub struct NodeGuard<B: Backend> {
    vnode: B::VNode,
    position: PositionIndices,
}
impl<B: Backend> Drop for NodeGuard<B> {
    fn drop(&mut self) {
        self.vnode.del_node(std::mem::take(&mut self.position));
    }
}
impl<B: Backend> NodeGuard<B> {
    pub(crate) fn new(vnode: B::VNode, position: PositionIndices) -> Self {
        Self { vnode, position }
    }
}
