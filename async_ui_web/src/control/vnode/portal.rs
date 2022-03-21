use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::{Rc, Weak},
};

use web_sys::Node;

use crate::control::{element_control::ElementControl, position::PositionIndices};

use super::{VNode, VNodeHandler};

#[derive(Debug)]
pub(crate) struct PortalVNode {
    inner: RefCell<PortalVNodeInner>,
}
impl PortalVNode {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(PortalVNodeInner {
                children: BTreeMap::new(),
                target: None,
            }),
        }
    }
    pub fn set_target(&self, control: &ElementControl) {
        let mut bm = self.inner.borrow_mut();
        if bm.target.is_some() {
            panic!("portal has more than one active exits");
        }
        let parent = &control.vnode;
        let position = control.position.clone();
        bm.target = Some((Rc::downgrade(parent), position.clone()));
        for (pos, nod) in bm.children.iter() {
            parent.ins_node(position.clone().merge(pos.clone()), nod.clone());
        }
    }
    pub fn unset_target(&self) {
        let mut bm = self.inner.borrow_mut();
        let (parent, position) = bm.target.take().expect("unset empty portal target");
        let parent = parent.upgrade().expect("portal target dropped prematurely");
        for pos in bm.children.keys() {
            parent.del_node(position.clone().merge(pos.clone()));
        }
    }
}
#[derive(Debug)]
struct PortalVNodeInner {
    children: BTreeMap<PositionIndices, Node>,
    target: Option<(Weak<VNode>, PositionIndices)>,
}
impl PortalVNodeInner {
    fn get_target(&self, position: PositionIndices) -> Option<(Rc<VNode>, PositionIndices)> {
        if let Some((wr, id)) = self.target.as_ref() {
            if let Some(parent) = wr.upgrade() {
                let new_pos = id.clone().merge(position);
                return Some((parent, new_pos));
            }
        }
        None
    }
}
impl VNodeHandler for PortalVNode {
    fn ins_node(&self, position: PositionIndices, node: Node) {
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
    fn del_node(&self, position: PositionIndices) -> Node {
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
