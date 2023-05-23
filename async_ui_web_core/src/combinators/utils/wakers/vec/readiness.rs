use super::super::dummy_waker;

use core::task::Waker;

use bitvec::vec::BitVec;

pub(crate) struct ReadinessVec {
    /// Whether each subfuture has woken.
    awake_set: BitVec<u8>,
    /// Indices of subfutures that have woken.
    awake_list: Vec<usize>,
    parent_waker: Waker,
}

impl ReadinessVec {
    pub(crate) fn new(len: usize) -> Self {
        let awake_set = BitVec::repeat(true, len);
        Self {
            awake_set,
            awake_list: (0..len).collect(),
            parent_waker: dummy_waker(),
        }
    }
    pub(crate) fn set_parent_waker(&mut self, waker: &Waker) {
        // If self.parent_waker and the given waker are the same then don't do the replacement.
        if !self.parent_waker.will_wake(waker) {
            self.parent_waker = waker.to_owned();
        }
    }
    fn set_woken(&mut self, index: usize) -> bool {
        let was_awake = self.awake_set.replace(index, true);
        if !was_awake {
            self.awake_list.push(index);
        }
        was_awake
    }
    pub(crate) fn wake(&mut self, index: usize) {
        if !self.set_woken(index) && self.awake_list.len() == 1 {
            self.parent_waker.wake_by_ref();
        }
    }
    pub(crate) fn awake_list(&self) -> &Vec<usize> {
        &self.awake_list
    }
    pub(crate) fn clear(&mut self) {
        // Depending on how many items was in the list,
        // either use `fill` (memset) or iterate and set each.
        // TODO: I came up with the 64 factor at random. Maybe test different factors?
        if self.awake_list.len() * 64 < self.awake_set.len() {
            let awake_set = &mut self.awake_set;
            self.awake_list.drain(..).for_each(|idx| {
                awake_set.set(idx, false);
            });
        } else {
            self.awake_list.clear();
            self.awake_set.fill(false);
        }
    }
}
