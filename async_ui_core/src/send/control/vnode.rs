use super::super::MaybeSend;
use super::position::PositionIndices;

pub trait VNode: Clone + MaybeSend + 'static {
    type Node;
    fn ins_node(&self, position: PositionIndices, node: Self::Node);
    fn del_node(&self, position: PositionIndices) -> Self::Node;
    fn move_node(&self, old_pos: PositionIndices, new_pos: PositionIndices) {
        let node = self.del_node(old_pos);
        self.ins_node(new_pos, node);
    }
}
