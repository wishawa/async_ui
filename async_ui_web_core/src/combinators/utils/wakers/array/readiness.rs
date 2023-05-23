use super::super::dummy_waker;

use core::task::Waker;

pub(crate) struct ReadinessArray<const N: usize> {
    /// Whether each subfuture has woken.
    awake_set: [bool; N],
    /// Indices of subfutures that have woken.
    awake_list: [usize; N],
    /// Number of subfutures that have woken.
    /// `awake_list` and `awake_list_len` together makes up something like ArrayVec<usize>.
    // TODO: Maybe just use the ArrayVec crate?
    awake_list_len: usize,
    parent_waker: Waker,
}

impl<const N: usize> ReadinessArray<N> {
    pub(crate) fn new() -> Self {
        Self {
            awake_set: [true; N],
            awake_list: core::array::from_fn(core::convert::identity),
            awake_list_len: N,
            parent_waker: dummy_waker(), // parent waker is dummy at first
        }
    }
    pub(crate) fn set_parent_waker(&mut self, waker: &Waker) {
        // If self.parent_waker and the given waker are the same then don't do the replacement.
        if !self.parent_waker.will_wake(waker) {
            self.parent_waker = waker.to_owned();
        }
    }
    fn set_woken(&mut self, index: usize) -> bool {
        let was_awake = std::mem::replace(&mut self.awake_set[index], true);
        if !was_awake {
            self.awake_list[self.awake_list_len] = index;
            self.awake_list_len += 1;
        }
        was_awake
    }
    pub(crate) fn wake(&mut self, index: usize) {
        if !self.set_woken(index) && self.awake_list_len == 1 {
            self.parent_waker.wake_by_ref();
        }
    }
    pub(crate) fn awake_list(&self) -> &[usize] {
        &self.awake_list[..self.awake_list_len]
    }
    const TRESHOLD: usize = N / 64;
    pub(crate) fn clear(&mut self) {
        // Depending on how many items was in the list,
        // either use `fill` (memset) or iterate and set each.
        // TODO: I came up with the 64 factor at random. Maybe test different factors?
        if self.awake_list_len < Self::TRESHOLD {
            self.awake_set.fill(false);
        } else {
            let awake_set = &mut self.awake_set;
            self.awake_list.iter().for_each(|&idx| {
                awake_set[idx] = false;
            });
        }
        self.awake_list_len = 0;
    }
}
