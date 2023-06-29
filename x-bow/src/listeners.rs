use std::{
    cell::{Cell, RefCell, RefMut},
    task::Waker,
};

/// Listeners associated with an edge.
pub struct Listener {
    wakers: RefCell<ListenerWakers>,
    up_version: Cell<u64>,
    down_version: Cell<u64>,
    here_version: Cell<u64>,
}
struct ListenerWakers {
    up_wakers: Vec<Waker>,
    down_wakers: Vec<Waker>,
    here_wakers: Vec<Waker>,
}

pub(crate) struct ListenerGroup<'a> {
    wakers: RefMut<'a, Vec<Waker>>,
    version: &'a Cell<u64>,
}
impl<'a> ListenerGroup<'a> {
    pub fn get_version(&mut self) -> u64 {
        self.version.get()
    }
    pub fn increment_version(&mut self) {
        self.version.set(self.version.get() + 1);
        self.wakers.drain(..).for_each(Waker::wake)
    }
    pub fn add_waker(&mut self, new_waker: &Waker, old_index: Option<usize>) -> usize {
        if let Some(idx) = old_index {
            if let Some(exisiting_waker) = self.wakers.get(idx) {
                if exisiting_waker.will_wake(new_waker) {
                    return idx;
                }
            }
        }
        let len = self.wakers.len();
        self.wakers.push(new_waker.to_owned());
        len
    }
}

impl Listener {
    pub const fn new() -> Self {
        let inner = RefCell::new(ListenerWakers {
            down_wakers: Vec::new(),
            up_wakers: Vec::new(),
            here_wakers: Vec::new(),
        });
        Self {
            wakers: inner,
            up_version: Cell::new(1),
            down_version: Cell::new(1),
            here_version: Cell::new(1),
        }
    }
    pub(crate) fn up(&self) -> ListenerGroup<'_> {
        ListenerGroup {
            wakers: RefMut::map(self.wakers.borrow_mut(), |l| &mut l.up_wakers),
            version: &self.up_version,
        }
    }
    pub(crate) fn down(&self) -> ListenerGroup<'_> {
        ListenerGroup {
            wakers: RefMut::map(self.wakers.borrow_mut(), |l| &mut l.down_wakers),
            version: &self.down_version,
        }
    }
    pub(crate) fn here(&self) -> ListenerGroup<'_> {
        ListenerGroup {
            wakers: RefMut::map(self.wakers.borrow_mut(), |l| &mut l.here_wakers),
            version: &self.here_version,
        }
    }
    pub fn invalidate_down(&self) {
        self.up().increment_version();
    }
}
