use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::{Rc, Weak},
};

use async_ui_core::local::control::{
    position::PositionIndices, vnode::VNode as VNodeTrait, Control,
};
use gtk::Widget;

use crate::backend::GtkBackend;

use super::{VNodeDispatch, VNodeEnum};

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
    pub fn set_target(&self, target: &Control<GtkBackend>) {
        let mut bm = self.inner.borrow_mut();
        if bm.target.is_some() {
            panic!("portal has more than one active exits");
        }
        let parent = target.get_vnode();
        let position = target.get_position();
        bm.target = Some((Rc::downgrade(&parent.0), position.clone()));
        for (pos, nod) in bm.children.iter() {
            parent.ins_node(position.clone().merge(pos.clone()), nod.clone());
        }
    }
    pub fn unset_target(&self) {
        let mut bm = self.inner.borrow_mut();
        let (parent, position) = bm.target.take().expect("unset empty portal target");
        let target = parent.upgrade().expect("portal target dropped prematurely");
        for pos in bm.children.keys() {
            target.dispatch_del_node(position.clone().merge(pos.clone()));
        }
    }
}
#[derive(Debug)]
struct PortalVNodeInner {
    children: BTreeMap<PositionIndices, Widget>,
    target: Option<(Weak<VNodeEnum>, PositionIndices)>,
}
impl PortalVNodeInner {
    fn get_target(&self, position: PositionIndices) -> Option<(Rc<VNodeEnum>, PositionIndices)> {
        if let Some((wr, id)) = self.target.as_ref() {
            if let Some(parent) = wr.upgrade() {
                let new_pos = id.clone().merge(position);
                return Some((parent, new_pos));
            }
        }
        None
    }
}
impl VNodeDispatch for PortalVNode {
    fn dispatch_ins_node(&self, position: PositionIndices, node: Widget) {
        let mut inner = self.inner.borrow_mut();
        if inner
            .children
            .insert(position.clone(), node.clone())
            .is_some()
        {
            panic!("more than one node added");
        }
        if let Some((parent, id)) = inner.get_target(position) {
            parent.dispatch_ins_node(id, node);
        }
    }
    fn dispatch_del_node(&self, position: PositionIndices) -> Widget {
        let mut inner = self.inner.borrow_mut();
        let node = inner
            .children
            .remove(&position)
            .expect("node not found for removal");
        if let Some((parent, id)) = inner.get_target(position) {
            parent.dispatch_del_node(id);
        }
        node
    }
}
