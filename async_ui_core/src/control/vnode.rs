use std::fmt::Debug;

use super::position::PositionIndices;
use super::Backend;
pub mod null;
pub mod portal;

pub trait VNode<B: Backend>: Debug + 'static {
	fn ins_node(&self, position: PositionIndices, node: B::NodeType);
	fn del_node(&self, position: PositionIndices) -> B::NodeType;
	fn move_node(&self, old_pos: PositionIndices, new_pos: PositionIndices) {
		let node = self.del_node(old_pos);
		self.ins_node(new_pos, node);
	}
}
