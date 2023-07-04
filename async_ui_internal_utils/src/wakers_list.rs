use std::{mem::replace, task::Waker};

use smallvec::SmallVec;

use crate::dummy_waker::dummy_waker;

pub struct WakersList {
    // Two linked lists in an Vec arena.
    // One is for free slots, other for used slots.
    // The first two items are the head of each linked list.
    items: SmallVec<[Item; 2]>,
}

const FREE_HEAD_INDEX: usize = 0;
const USED_HEAD_INDEX: usize = 1;

#[derive(Debug, Clone)]
struct Item {
    waker: Waker,
    prev: usize,
    next: usize,
}

pub struct WakerSlot(usize);
impl WakerSlot {
    pub const INVALID: Self = Self(usize::MAX);
}

impl WakersList {
    pub fn new() -> Self {
        Self {
            items: smallvec::smallvec![
                Item {
                    waker: dummy_waker(),
                    next: 0,
                    prev: 0
                },
                Item {
                    waker: dummy_waker(),
                    next: 1,
                    prev: 1
                }
            ],
        }
    }
    pub fn add(&mut self) -> WakerSlot {
        let free_head = &mut self.items[FREE_HEAD_INDEX];
        // Is the free list empty?
        let (index, must_fix_free_list) = if free_head.next == FREE_HEAD_INDEX {
            (
                self.grow_full_list(), // not enough capacity, grow the list
                false,                 // in this case the node was never in the free list
            )
        } else {
            (
                free_head.next, // the first item in the free list
                true, // need to fix the free list afterward because we just took something from it
            )
        };
        // add to used list
        let used_head = &mut self.items[USED_HEAD_INDEX];
        let old_used_prev = replace(&mut used_head.prev, index);
        self.items[old_used_prev].next = index;
        let node = &mut self.items[index];
        let (old_free_prev, old_free_next) = (
            replace(&mut node.prev, old_used_prev),
            replace(&mut node.next, USED_HEAD_INDEX),
        );
        // fix the free list
        if must_fix_free_list {
            self.items[old_free_prev].next = old_free_next;
            self.items[old_free_next].prev = old_free_prev;
        }
        WakerSlot(index)
    }
    pub fn remove(&mut self, handle: &WakerSlot) {
        let &WakerSlot(index) = handle;
        // add to free list
        let free_head = &mut self.items[FREE_HEAD_INDEX];
        let old_free_next = replace(&mut free_head.next, index);
        self.items[old_free_next].prev = index;
        let node = &mut self.items[index];
        let (old_used_prev, old_used_next) = (
            replace(&mut node.prev, FREE_HEAD_INDEX),
            replace(&mut node.next, old_free_next),
        );
        node.waker = dummy_waker();
        // fix the used list
        self.items[old_used_prev].next = old_used_next;
        self.items[old_used_next].prev = old_used_prev;
    }
    pub fn update(&mut self, slot: &WakerSlot, waker: &Waker) {
        let target = &mut self.items[slot.0].waker;
        if !target.will_wake(waker) {
            *target = waker.to_owned();
        }
    }
    pub fn iter<'s>(&'s self) -> impl Iterator<Item = &'s Waker> + 's {
        let mut index = self.items[USED_HEAD_INDEX].next;
        std::iter::from_fn(move || {
            if index != USED_HEAD_INDEX {
                let item = &self.items[index];
                index = item.next;
                Some(&item.waker)
            } else {
                None
            }
        })
    }
    /// Double the capacity of the list and return the index of a free, disconnected node.
    fn grow_full_list(&mut self) -> usize {
        let old_len = self.items.len();
        self.items.extend((old_len..(old_len * 2)).map(|idx| Item {
            waker: dummy_waker(),
            prev: idx - 1,
            next: idx + 1,
        }));
        self.items[old_len + 1].prev = FREE_HEAD_INDEX;
        self.items[old_len * 2 - 1].next = FREE_HEAD_INDEX;
        let free_head = &mut self.items[FREE_HEAD_INDEX];
        free_head.prev = old_len * 2 - 1;
        free_head.next = old_len + 1;
        old_len
    }
}
