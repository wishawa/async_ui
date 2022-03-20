use web_sys::Node;
mod node;
mod null;
mod portal;
pub(crate) use node::NodeVNode;
pub(crate) use null::NullVNode;
pub(crate) use portal::PortalVNode;

use super::position::PositionIndices;

#[enum_dispatch::enum_dispatch]
pub(crate) enum VNode {
    NodeVNode,
    PortalVNode,
    NullVNode,
}
#[enum_dispatch::enum_dispatch(VNode)]
pub(crate) trait VNodeHandler {
    fn ins_node(&self, position: PositionIndices, node: Node);
    fn del_node(&self, position: PositionIndices) -> Node;
}

impl VNode {
    pub(crate) fn dummy() -> Self {
        Self::from(NullVNode)
    }
}
