use std::rc::Rc;

use super::{
    position::PositionIndices,
    vnode::{VNode, VNodeHandler},
};

pub struct NodeGuard {
    vnode: Rc<VNode>,
    position: PositionIndices,
}
impl Drop for NodeGuard {
    fn drop(&mut self) {
        self.vnode.del_node(std::mem::take(&mut self.position));
    }
}
impl NodeGuard {
    pub(crate) fn new(vnode: Rc<VNode>, position: PositionIndices) -> Self {
        Self { vnode, position }
    }
}
