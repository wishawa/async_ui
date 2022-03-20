use std::rc::Rc;

use super::node_guard::NodeGuard;
use super::position::PositionIndices;
use super::vnode::{VNode, VNodeHandler};
use web_sys::Node;
thread_local! {
    static DUMMY_ELEMENT_CONTROL: ElementControl = ElementControl {
        vnode: Rc::new(VNode::dummy()),
        position: PositionIndices::default(),
    }
}
scoped_tls::scoped_thread_local! {
    pub(crate) static ELEMENT_CONTROL: ElementControl
}
#[derive(Clone)]
pub(crate) struct ElementControl {
    pub(in crate::control) vnode: Rc<VNode>,
    pub(in crate::control) position: PositionIndices,
}
impl ElementControl {
    pub fn get_dummy() -> Self {
        DUMMY_ELEMENT_CONTROL.with(Self::clone)
    }
    pub fn nest(&self, index: usize) -> Self {
        let mut new = self.clone();
        new.position.nest(index);
        new
    }
    pub fn put_node(&self, node: Node) -> NodeGuard {
        self.vnode.ins_node(self.position.clone(), node);
        NodeGuard::new(self.vnode.clone(), self.position.clone())
    }
    pub fn new_with_vnode(vnode: Rc<VNode>) -> Self {
        Self {
            vnode,
            position: PositionIndices::default(),
        }
    }
}
