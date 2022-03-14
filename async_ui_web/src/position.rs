use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use smallvec::SmallVec;
use web_sys::Node;

type PositionIndices = SmallVec<[usize; 8]>;

#[derive(Debug)]
pub(crate) struct Position {
    siblings: Rc<SiblingNodes>,
    indices: PositionIndices,
}

impl Position {
    pub fn new_in_node(node: Node) -> Self {
        Self {
            siblings: Rc::new(RefCell::new(SiblingNodesInner {
                parent: node,
                map: BTreeMap::new(),
            })),
            indices: PositionIndices::new(),
        }
    }
    pub fn nest_fragment(&self, child_index: usize) -> Self {
        let mut indices = PositionIndices::with_capacity(self.indices.len() + 1);
        indices.clone_from(&self.indices);
        indices.push(child_index);
        let siblings = self.siblings.clone();
        Self { indices, siblings }
    }
    pub fn add_node(&self, node: Node) {
        let mut bm = self.siblings.borrow_mut();
        let next_node = bm.map.range(self.indices.clone()..).next().map(|(_, v)| v);
        bm.parent
            .insert_before(&node, next_node)
            .expect("node insertion failed");
        bm.map.insert(self.indices.clone(), node);
    }
}
impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        (self.indices == other.indices) && (Rc::ptr_eq(&self.siblings, &other.siblings))
    }
}
impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if Rc::ptr_eq(&self.siblings, &other.siblings) {
            Some(self.indices.cmp(&other.indices))
        } else {
            None
        }
    }
}
impl Eq for Position {}
impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other)
            .expect("siblings have different parents")
    }
}

impl Drop for Position {
    fn drop(&mut self) {
        let mut bm = self.siblings.borrow_mut();
        if let Some(node) = bm.map.remove(&self.indices) {
            bm.parent.remove_child(&node).ok();
        }
    }
}

pub(crate) type SiblingNodes = RefCell<SiblingNodesInner>;

#[derive(Debug)]
pub(crate) struct SiblingNodesInner {
    parent: Node,
    map: BTreeMap<PositionIndices, Node>,
}
