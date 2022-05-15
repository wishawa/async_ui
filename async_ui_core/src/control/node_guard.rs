use super::super::backend::Backend;

use super::position::PositionIndices;
use super::VNodeWrap;

pub struct NodeGuard<B: Backend> {
	vnode: VNodeWrap<B>,
	position: PositionIndices,
}
impl<B: Backend> Drop for NodeGuard<B> {
	fn drop(&mut self) {
		self.vnode.del_node(std::mem::take(&mut self.position));
	}
}
impl<B: Backend> NodeGuard<B> {
	pub(crate) fn new(vnode: VNodeWrap<B>, position: PositionIndices) -> Self {
		Self { vnode, position }
	}
}
