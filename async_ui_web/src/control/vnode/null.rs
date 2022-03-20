use web_sys::Node;

use crate::control::position::PositionIndices;

use super::VNodeHandler;

pub(crate) struct NullVNode;
impl VNodeHandler for NullVNode {
    fn ins_node(&self, _position: PositionIndices, _node: Node) {
        panic!("Null VNode called");
    }
    fn del_node(&self, _position: PositionIndices) -> Node {
        panic!("Null VNode called");
    }
}
