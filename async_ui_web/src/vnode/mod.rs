use std::rc::Rc;

use async_ui_core::control::{position::PositionIndices, vnode::VNode as VNodeTrait};
use web_sys::Node;
mod node;
mod null;
mod portal;
pub(crate) use node::NodeVNode;
pub(crate) use null::NullVNode;
pub(crate) use portal::PortalVNode;

#[enum_dispatch::enum_dispatch]
#[derive(Debug)]
pub(crate) enum VNodeEnum {
    NodeVNode,
    PortalVNode,
    NullVNode,
}
#[enum_dispatch::enum_dispatch(VNodeEnum)]
trait VNodeDispatch {
    fn dispatch_ins_node(&self, position: PositionIndices, node: Node);
    fn dispatch_del_node(&self, position: PositionIndices) -> Node;
}

#[derive(Debug, Clone)]
pub struct VNode(pub(crate) Rc<VNodeEnum>);

impl VNodeTrait for VNode {
    type Node = Node;
    fn ins_node(&self, position: PositionIndices, node: Self::Node) {
        self.0.dispatch_ins_node(position, node)
    }

    fn del_node(&self, position: PositionIndices) -> Self::Node {
        self.0.dispatch_del_node(position)
    }
}
