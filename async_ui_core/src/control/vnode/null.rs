use super::{Backend, PositionIndices, VNode};
#[derive(Debug)]
pub struct NullVNode;
impl<B: Backend> VNode<B> for NullVNode {
    fn ins_node(&self, _position: PositionIndices, _node: <B as Backend>::NodeType) {
        panic!("NullVNode used")
    }

    fn del_node(&self, _position: PositionIndices) -> <B as Backend>::NodeType {
        panic!("NullVNode used")
    }
}
