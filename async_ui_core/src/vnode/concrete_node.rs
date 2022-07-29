use std::{cell::RefCell, collections::BTreeMap};

use crate::{backend::BackendTrait, position::PositionIndex};

use super::VNodeTrait;

pub struct ConcreteNodeVNode<B: BackendTrait> {
    node: RefNode<B>,
    children: RefCell<BTreeMap<PositionIndex, B::Node>>,
}
enum RefNode<B: BackendTrait> {
    Parent { parent: B::Node },
    Sibling { parent: B::Node, sibling: B::Node },
}

impl<B: BackendTrait> VNodeTrait<B> for ConcreteNodeVNode<B> {
    fn add_child_node(&self, node: <B as BackendTrait>::Node, position: PositionIndex) {
        let mut children_map = self.children.borrow_mut();
        let next_node = children_map
            .range(position.clone()..)
            .next()
            .map(|(_k, v)| v);
        match &self.node {
            RefNode::Parent { parent } => {
                B::add_child_node(parent, &node, next_node);
            }
            RefNode::Sibling { parent, sibling } => {
                B::add_child_node(parent, &node, Some(next_node.unwrap_or(sibling)));
            }
        }
        children_map.insert(position, node);
    }

    fn del_child_node(&self, position: PositionIndex) {
        let mut children_map = self.children.borrow_mut();
        let removed = children_map.remove(&position);
        if let Some(removed) = removed {
            match &self.node {
                RefNode::Parent { parent } => B::del_child_node(parent, &removed),
                RefNode::Sibling { parent, .. } => B::del_child_node(parent, &removed),
            }
        }
    }
}
