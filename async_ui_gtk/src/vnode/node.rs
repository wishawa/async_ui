use std::{cell::RefCell, collections::BTreeMap};

use async_ui_core::local::control::position::PositionIndices;
use gtk::{traits::WidgetExt, Widget};

use super::VNodeDispatch;

#[derive(Debug)]
pub(crate) struct NodeVNode {
    node: Widget,
    children: RefCell<BTreeMap<PositionIndices, Widget>>,
}
impl VNodeDispatch for NodeVNode {
    fn dispatch_ins_node(&self, position: PositionIndices, node: Widget) {
        let mut bm = self.children.borrow_mut();
        let next_node = bm.range(position.clone()..).next().map(|(_k, v)| v);
        println!("inserting node, next node exists = {}", next_node.is_some());
        node.insert_before(&self.node, next_node);
        if bm.insert(position, node).is_some() {
            panic!("more than one node added");
        }
    }
    fn dispatch_del_node(&self, position: PositionIndices) -> Widget {
        let mut bm = self.children.borrow_mut();
        let node = bm.remove(&position).expect("node not found for removal");
        node.unparent();
        node
    }
}
impl NodeVNode {
    pub fn new(node: Widget) -> Self {
        Self {
            node,
            children: Default::default(),
        }
    }
}
