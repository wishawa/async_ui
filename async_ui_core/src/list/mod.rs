use std::{
    cell::Cell,
    collections::VecDeque,
    ops::{Bound, Range, RangeBounds},
};

use im_rc::Vector;

#[derive(Clone)]
pub enum Change<T> {
    Splice {
        remove_range: Range<usize>,
        replace_with: Vector<T>,
    },
    Remove {
        index: usize,
    },
    Insert {
        index: usize,
        value: T,
    },
}
pub struct ListModel<T: Clone> {
    head: Vector<T>,
    log: VecDeque<Change<T>>,
    log_start_version: u64,
    total_listeners: Cell<usize>,
    pending_listeners: Cell<usize>,
}

// taken from https://users.rust-lang.org/t/whats-the-best-way-to-convert-from-rangebounds-to-range/31607
fn bound_to_idx(
    (start, end): (Bound<&usize>, Bound<&usize>),
    min: usize,
    max: usize,
) -> Range<usize> {
    let start = match start {
        Bound::Included(start) => *start,
        Bound::Excluded(start) => start + 1,
        Bound::Unbounded => min,
    };
    let end = match end {
        Bound::Included(end) => end + 1,
        Bound::Excluded(end) => *end,
        Bound::Unbounded => max,
    };
    start..end
}
fn apply_change<T: Clone>(vector: &mut Vector<T>, change: Change<T>) {
    match change {
        Change::Splice {
            remove_range,
            replace_with,
        } => {
            let n_items = ExactSizeIterator::len(&remove_range);
            let mut right = vector.split_off(remove_range.start);
            let right = right.split_off(n_items);
            vector.append(replace_with);
            vector.append(right);
        }
        Change::Remove { index } => {
            vector.remove(index);
        }
        Change::Insert { index, value } => {
            vector.insert(index, value);
        }
    }
}
impl<T: Clone> ListModel<T> {
    pub fn new() -> Self {
        Self::from_iter([].into_iter())
    }
    pub fn from_iter<I: Iterator<Item = T>>(iter: I) -> Self {
        Self {
            head: iter.collect(),
            log: VecDeque::new(),
            log_start_version: 0,
            total_listeners: Cell::new(0),
            pending_listeners: Cell::new(0),
        }
    }
    fn change(&mut self, change: Change<T>) {
        let total_listeners = self.total_listeners.get();
        if total_listeners == 0 {
            apply_change(&mut self.head, change);
            self.log_start_version += self.log.len() as u64;
            self.log.clear();
        } else {
            apply_change(&mut self.head, change.clone());
            if self.pending_listeners.get() == 0 {
                self.log_start_version += self.log.len() as u64;
                self.log.clear();
            } else {
                apply_change(&mut self.head, change.clone());
            }
            self.log.push_back(change);
            self.pending_listeners.set(total_listeners);
        }
    }
    pub fn insert(&mut self, index: usize, value: T) {
        self.change(Change::Insert { index, value })
    }
    pub fn remove(&mut self, index: usize) {
        self.change(Change::Remove { index })
    }
    pub fn splice<R: RangeBounds<usize>, I: Iterator<Item = T>>(
        &mut self,
        remove_range: R,
        replace_with: I,
    ) {
        let remove_range = bound_to_idx(
            (remove_range.start_bound(), remove_range.end_bound()),
            0,
            self.head.len(),
        );
        self.change(Change::Splice {
            remove_range,
            replace_with: replace_with.collect(),
        })
    }
    pub fn push(&mut self, value: T) {
        self.change(Change::Insert {
            index: self.head.len(),
            value,
        });
    }
    pub fn pop(&mut self) {
        self.change(Change::Remove {
            index: self.head.len() - 1,
        });
    }
    pub fn underlying_vector(&self) -> &Vector<T> {
        &self.head
    }
}

pub struct ListModelPrivateAPIs<'l, T: Clone>(pub &'l ListModel<T>);

impl<'l, T: Clone> ListModelPrivateAPIs<'l, T> {
    pub fn get_version(&self) -> u64 {
        self.0.log_start_version + self.0.log.len() as u64
    }
    pub fn changes_since_version(
        &self,
        version: u64,
    ) -> std::collections::vec_deque::Iter<'_, Change<T>> {
        let min = (version - self.0.log_start_version) as usize;
        self.0.log.range(min..)
    }
    pub fn total_listeners(&self) -> &'_ Cell<usize> {
        &self.0.total_listeners
    }
    pub fn pending_listeners(&self) -> &'_ Cell<usize> {
        &self.0.pending_listeners
    }
}
