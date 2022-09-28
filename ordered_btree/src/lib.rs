use std::{
    cell::{Ref, RefCell, RefMut},
    rc::{Rc, Weak},
};

use arrayvec::ArrayVec;

pub struct Root<V, const BP: usize> {
    root: RefCell<Rc<RefCell<Chunk<V, BP>>>>,
}
struct Chunk<V, const BP: usize> {
    edges: Edges<V, BP>,
    parent: Option<Weak<RefCell<Chunk<V, BP>>>>,
}
struct LeafNode<V, const BP: usize> {
    value: V,
    parent: Weak<RefCell<Chunk<V, BP>>>,
}

enum Edges<V, const BP: usize> {
    Chunks {
        chunks: ArrayVec<Rc<RefCell<Chunk<V, BP>>>, BP>,
        counts: ArrayVec<usize, BP>,
    },
    Leaves {
        leaves: ArrayVec<Rc<RefCell<LeafNode<V, BP>>>, BP>,
    },
}

impl<V, const BP: usize> Edges<V, BP> {
    fn set_parent(&mut self, parent: Weak<RefCell<Chunk<V, BP>>>) {
        match self {
            Edges::Chunks { chunks, counts } => {
                for chunk in chunks.iter() {
                    chunk.borrow_mut().parent = Some(parent.clone());
                }
            }
            Edges::Leaves { leaves } => {
                for leaf in leaves.iter() {
                    leaf.borrow_mut().parent = parent.clone();
                }
            }
        }
    }
    fn get_count(&self) -> usize {
        match self {
            Edges::Chunks { counts, .. } => counts.iter().sum(),
            Edges::Leaves { leaves } => leaves.len(),
        }
    }
    fn get_length(&self) -> usize {
        match self {
            Edges::Chunks { counts, .. } => counts.len(),
            Edges::Leaves { leaves } => leaves.len(),
        }
    }
}

const VIOL_CONNECTION: &str = "invariant violation: parent-child connection";
const VIOL_LEAF_DEPTH: &str = "invariant violation: uniform leaf depth";
pub struct Cursor<V, const BP: usize> {
    node: Rc<RefCell<LeafNode<V, BP>>>,
    root: Rc<Root<V, BP>>,
}
fn search_counts(counts: &[usize], index: usize) -> Option<(usize, usize)> {
    let mut acc = 0;
    for (i, count) in counts.iter().enumerate() {
        acc += count;
        if acc > index {
            return Some((i, (index + count) - acc));
        }
    }
    return None;
}

impl<V, const BP: usize> Cursor<V, BP> {
    pub fn insert_before(&self, value: V) {
        self.insert_with_offset(value, 0)
    }
    pub fn insert_after(&self, value: V) {
        self.insert_with_offset(value, 1)
    }
    fn insert_with_offset(&self, value: V, offset: usize) {
        let chunk_rc = self.node.borrow().parent.upgrade().expect(VIOL_CONNECTION);
        let mut chunk = chunk_rc.borrow_mut();
        match &mut chunk.edges {
            Edges::Leaves { leaves } => {
                let pos = leaves
                    .iter()
                    .position(|leaf| Rc::ptr_eq(&self.node, leaf))
                    .expect(VIOL_CONNECTION);
                let new_leaf = LeafNode {
                    parent: Rc::downgrade(&chunk_rc),
                    value,
                };
                let new_leaf = Rc::new(RefCell::new(new_leaf));
                leaves.insert(pos + offset, new_leaf);
                let leaves_len = leaves.len();
                drop(chunk);
                self.root.modify_count(&chunk_rc, 1);
                if leaves_len == BP {
                    self.root.rebalance_split(&chunk_rc)
                }
            }
            Edges::Chunks { .. } => {
                unreachable!("{}", VIOL_LEAF_DEPTH)
            }
        }
    }
    pub fn remove(self) {
        let chunk_rc = self.node.borrow().parent.upgrade().expect(VIOL_CONNECTION);
        let mut chunk = chunk_rc.borrow_mut();
        match &mut chunk.edges {
            Edges::Leaves { leaves } => {
                let pos = leaves
                    .iter()
                    .position(|leaf| Rc::ptr_eq(&self.node, leaf))
                    .expect(VIOL_CONNECTION);
                leaves.remove(pos);
                let leaves_len = leaves.len();
                drop(chunk);
                self.root.modify_count(&chunk_rc, -1);
                if leaves_len < (BP / 2) {
                    self.root.rebalance_merge(&chunk_rc);
                }
            }
            Edges::Chunks { .. } => unreachable!("{}", VIOL_LEAF_DEPTH),
        }
    }
    pub fn value<'b>(&'b self) -> Ref<'b, V> {
        Ref::map(self.node.borrow(), |n| &n.value)
    }
    pub fn value_mut<'b>(&'b self) -> RefMut<'b, V> {
        RefMut::map(self.node.borrow_mut(), |n| &mut n.value)
    }
}

impl<V, const BP: usize> Root<V, BP> {
    pub fn cursor(self: &Rc<Self>, index: usize) -> Cursor<V, BP> {
        self.search(&*self.root.borrow(), index)
    }
}
impl<V, const BP: usize> Root<V, BP> {
    const HALF: usize = BP / 2;
    fn search(self: &Rc<Self>, this: &Rc<RefCell<Chunk<V, BP>>>, index: usize) -> Cursor<V, BP> {
        let me = &*this.borrow();
        match &me.edges {
            Edges::Chunks {
                chunks: edges,
                counts,
            } => {
                let (pos, inner_index) = search_counts(&*counts, index).expect("out of bound");
                self.search(&edges[pos], inner_index)
            }
            Edges::Leaves { leaves } => Cursor {
                node: leaves[index].to_owned(),
                root: self.clone(),
            },
        }
    }

    fn modify_count(self: &Rc<Self>, this: &Rc<RefCell<Chunk<V, BP>>>, delta: isize) {
        if let Some(parent) = this
            .borrow()
            .parent
            .as_ref()
            .map(|p| p.upgrade().expect(VIOL_CONNECTION))
        {
            let mut bm = parent.borrow_mut();
            match &mut bm.edges {
                Edges::Chunks { chunks, counts } => {
                    let pos = chunks
                        .iter()
                        .position(|chunk| Rc::ptr_eq(this, chunk))
                        .expect(VIOL_CONNECTION);
                    let count = counts
                        .get_mut(pos)
                        .expect("invariant violation: counts-edges parallelism");
                    *count = ((*count as isize) + delta) as usize;
                }
                Edges::Leaves { .. } => unreachable!("{}", VIOL_CONNECTION),
            }
            self.modify_count(&parent, delta);
        }
    }

    fn rebalance_split(self: &Rc<Self>, this: &Rc<RefCell<Chunk<V, BP>>>) {
        let mut me = this.borrow_mut();
        let new_edges = match &mut me.edges {
            Edges::Chunks { chunks, counts } => {
                let counts: ArrayVec<usize, BP> = counts.drain(Self::HALF..).collect();
                Edges::Chunks {
                    chunks: chunks.drain(Self::HALF..).collect(),
                    counts,
                }
            }
            Edges::Leaves { leaves } => {
                let leaves: ArrayVec<Rc<RefCell<LeafNode<V, BP>>>, BP> =
                    leaves.drain(Self::HALF..).collect();
                Edges::Leaves { leaves }
            }
        };
        let new_count = new_edges.get_count();

        let new_chunk = Chunk {
            edges: new_edges,
            parent: me.parent.clone(),
        };
        let new_chunk = Rc::new(RefCell::new(new_chunk));
        let new_chunk_weak = Rc::downgrade(&new_chunk);
        new_chunk.borrow_mut().edges.set_parent(new_chunk_weak);

        if let Some(parent_rc) = me
            .parent
            .as_ref()
            .map(|p| p.upgrade().expect(VIOL_CONNECTION))
        {
            let mut bm = parent_rc.borrow_mut();
            match &mut bm.edges {
                Edges::Chunks { chunks, counts } => {
                    let pos = chunks
                        .iter()
                        .position(|chunk| Rc::ptr_eq(this, chunk))
                        .expect(VIOL_CONNECTION);
                    chunks.insert(pos, new_chunk);
                    counts[pos] -= new_count;
                    counts.insert(pos, new_count);
                    if chunks.len() == BP {
                        self.rebalance_split(&parent_rc);
                    }
                }
                Edges::Leaves { .. } => unreachable!("{}", VIOL_LEAF_DEPTH),
            }
        } else {
            let new_root = Chunk {
                edges: Edges::Chunks {
                    chunks: [this.clone(), new_chunk].into_iter().collect(),
                    counts: [me.edges.get_count(), new_count].into_iter().collect(),
                },
                parent: None,
            };
            let new_root = Rc::new(RefCell::new(new_root));
            let new_root_weak = Rc::downgrade(&new_root);
            new_root.borrow_mut().edges.set_parent(new_root_weak);
            *self.root.borrow_mut() = new_root;
        }
    }
    fn rebalance_merge(self: &Rc<Self>, this: &Rc<RefCell<Chunk<V, BP>>>) {
        let mut me = this.borrow_mut();
        let ml = me.edges.get_length();

        if ml >= Self::HALF {
            return;
        }

        if let Some(parent_rc) = me
            .parent
            .as_ref()
            .map(|p| p.upgrade().expect(VIOL_CONNECTION))
        {
            let mut bm = parent_rc.borrow_mut();
            match &mut bm.edges {
                Edges::Chunks { chunks, counts } => {
                    let pos = chunks
                        .iter()
                        .position(|chunk| Rc::ptr_eq(this, chunk))
                        .expect(VIOL_CONNECTION);
                    enum Direction {
                        Previous,
                        Next,
                    }
                    fn transfer_or_merge<V, const BP: usize>(
                        from_rc: &Rc<RefCell<Chunk<V, BP>>>,
                        to_rc: &Rc<RefCell<Chunk<V, BP>>>,
                        from: &mut Chunk<V, BP>,
                        to: &mut Chunk<V, BP>,
                        parent_counts: &mut ArrayVec<usize, BP>,
                        parent_chunks: &mut ArrayVec<Rc<RefCell<Chunk<V, BP>>>, BP>,
                        pos: usize,
                        direction: Direction,
                    ) {
                        let sib_pos = match direction {
                            Direction::Previous => pos - 1,
                            Direction::Next => pos + 1,
                        };
                        if from.edges.get_length() > (BP / 2) {
                            let (from_idx, to_idx) = match direction {
                                Direction::Previous => (from.edges.get_length() - 1, 0),
                                Direction::Next => (0, to.edges.get_length()),
                            };
                            let moved = match (&mut from.edges, &mut to.edges) {
                                (
                                    Edges::Chunks {
                                        chunks: sib_chunks,
                                        counts: sib_counts,
                                    },
                                    Edges::Chunks {
                                        chunks: my_chunks,
                                        counts: my_counts,
                                    },
                                ) => {
                                    let ch = sib_chunks.remove(from_idx);
                                    let co = sib_counts.remove(from_idx);
                                    my_chunks.insert(to_idx, ch);
                                    my_counts.insert(to_idx, co);
                                    co
                                }
                                (
                                    Edges::Leaves { leaves: sib_leaves },
                                    Edges::Leaves { leaves: my_leaves },
                                ) => {
                                    let le = sib_leaves.remove(from_idx);
                                    my_leaves.insert(to_idx, le);
                                    1
                                }
                                _ => unreachable!("{}", VIOL_LEAF_DEPTH),
                            };
                            parent_counts[sib_pos] -= moved;
                            parent_counts[pos] += moved;
                        } else {
                            let (
                                merge_target_rc,
                                _merge_source_rc,
                                merge_target,
                                merge_source,
                                target_pos,
                                source_pos,
                            ) = match direction {
                                Direction::Previous => (from_rc, to_rc, from, to, sib_pos, pos),
                                Direction::Next => (to_rc, from_rc, to, from, pos, sib_pos),
                            };
                            merge_source
                                .edges
                                .set_parent(Rc::downgrade(merge_target_rc));
                            match (&mut merge_target.edges, &mut merge_source.edges) {
                                (
                                    Edges::Chunks {
                                        chunks: target_chunks,
                                        counts: target_counts,
                                    },
                                    Edges::Chunks {
                                        chunks: source_chunks,
                                        counts: source_counts,
                                    },
                                ) => {
                                    target_chunks.extend(source_chunks.drain(..));
                                    target_counts.extend(source_counts.drain(..));
                                }
                                (
                                    Edges::Leaves {
                                        leaves: target_leaves,
                                    },
                                    Edges::Leaves {
                                        leaves: source_leaves,
                                    },
                                ) => target_leaves.extend(source_leaves.drain(..)),
                                _ => unreachable!("{}", VIOL_LEAF_DEPTH),
                            }
                            parent_counts[target_pos] += parent_counts[source_pos];
                            parent_counts.remove(source_pos);
                            parent_chunks.remove(source_pos);
                        }
                    }
                    let next_chunk = chunks.get(pos + 1).cloned();
                    let last_chunk = (if pos > 0 { chunks.get(pos - 1) } else { None }).cloned();
                    if let Some(next_chunk) = next_chunk {
                        let mut next_sib = next_chunk.borrow_mut();
                        transfer_or_merge(
                            &next_chunk,
                            this,
                            &mut *next_sib,
                            &mut *me,
                            counts,
                            chunks,
                            pos,
                            Direction::Next,
                        );
                        self.rebalance_merge(&parent_rc);
                        return;
                    }
                    if let Some(last_chunk) = last_chunk {
                        let mut last_sib = last_chunk.borrow_mut();
                        transfer_or_merge(
                            &last_chunk,
                            this,
                            &mut *last_sib,
                            &mut *me,
                            counts,
                            chunks,
                            pos,
                            Direction::Previous,
                        );
                        self.rebalance_merge(&parent_rc);
                        return;
                    }
                    unreachable!("invariant violation: array length");
                }
                Edges::Leaves { .. } => unreachable!("{}", VIOL_LEAF_DEPTH),
            }
        }
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
