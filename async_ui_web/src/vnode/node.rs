use std::{cell::RefCell, collections::BTreeMap};

use async_ui_core::local::control::{position::PositionIndices, vnode::VNode};
use web_sys::Node;

use crate::manual_apis::WebBackend;

#[derive(Debug)]
pub(crate) struct NodeVNode {
    node: Node,
    children: RefCell<BTreeMap<PositionIndices, Node>>,
}
impl VNode<WebBackend> for NodeVNode {
    fn ins_node(&self, position: PositionIndices, node: Node) {
        let mut bm = self.children.borrow_mut();
        let next_node = bm.range(position.clone()..).next().map(|(_k, v)| v);
        self.node
            .insert_before(&node, next_node)
            .expect("node insertion failed");
        if bm.insert(position, node).is_some() {
            panic!("more than one node added");
        }
    }
    fn del_node(&self, position: PositionIndices) -> Node {
        let mut bm = self.children.borrow_mut();
        let node = bm.remove(&position).expect("node not found for removal");
        self.node.remove_child(&node).expect("node removal failed");
        node
    }
}
impl NodeVNode {
    pub fn new(node: Node) -> Self {
        Self {
            node,
            children: Default::default(),
        }
    }
}
