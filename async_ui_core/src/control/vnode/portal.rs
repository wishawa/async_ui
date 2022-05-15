use std::{
	cell::RefCell,
	collections::BTreeMap,
	rc::{Rc, Weak},
};

use super::{
	super::{Control, PositionIndices},
	Backend, VNode,
};

pub struct PortalVNode<B: Backend> {
	inner: RefCell<PortalVNodeInner<B>>,
}
impl<B: Backend> PortalVNode<B> {
	pub fn new() -> Self {
		Self {
			inner: RefCell::new(PortalVNodeInner {
				children: BTreeMap::new(),
				target: None,
			}),
		}
	}
	pub fn set_target(&self, target: &Control<B>) {
		let mut bm = self.inner.borrow_mut();
		if bm.target.is_some() {
			panic!("portal has more than one active exits");
		}
		let parent = target.get_vnode();
		let position = target.get_position();
		bm.target = Some((Rc::downgrade(parent), position.clone()));
		for (pos, nod) in bm.children.iter() {
			parent.ins_node(position.clone().merge(pos.clone()), nod.clone());
		}
	}
	pub fn unset_target(&self) {
		let mut bm = self.inner.borrow_mut();
		let (parent, position) = bm.target.take().expect("unset empty portal target");
		let target = parent.upgrade().expect("portal target dropped prematurely");
		for pos in bm.children.keys() {
			target.del_node(position.clone().merge(pos.clone()));
		}
	}
}
struct PortalVNodeInner<B: Backend> {
	children: BTreeMap<PositionIndices, B::NodeType>,
	target: Option<(Weak<dyn VNode<B>>, PositionIndices)>,
}
impl<B: Backend> PortalVNodeInner<B> {
	fn get_target(&self, position: PositionIndices) -> Option<(Rc<dyn VNode<B>>, PositionIndices)> {
		if let Some((wr, id)) = self.target.as_ref() {
			if let Some(parent) = wr.upgrade() {
				let new_pos = id.clone().merge(position);
				return Some((parent, new_pos));
			}
		}
		None
	}
}
impl<B: Backend> VNode<B> for PortalVNode<B> {
	fn ins_node(&self, position: PositionIndices, node: B::NodeType) {
		let mut inner = self.inner.borrow_mut();
		if inner
			.children
			.insert(position.clone(), node.clone())
			.is_some()
		{
			panic!("more than one node added");
		}
		if let Some((parent, id)) = inner.get_target(position) {
			parent.ins_node(id, node);
		}
	}
	fn del_node(&self, position: PositionIndices) -> B::NodeType {
		let mut inner = self.inner.borrow_mut();
		let node = inner
			.children
			.remove(&position)
			.expect("node not found for removal");
		if let Some((parent, id)) = inner.get_target(position) {
			parent.del_node(id);
		}
		node
	}
}

impl<B: Backend> std::fmt::Debug for PortalVNode<B> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("PortalVNode").finish()
	}
}
