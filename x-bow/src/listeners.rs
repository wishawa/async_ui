use std::{
    cell::{Cell, RefCell, RefMut},
    task::Waker,
};

use async_ui_internal_utils::wakers_list::WakersList;

/// Listeners associated with an edge.
pub struct Listener {
    wakers: RefCell<ListenerWakers>,
    up_version: Cell<u64>,
    down_version: Cell<u64>,
    here_version: Cell<u64>,
}
struct ListenerWakers {
    up_wakers: WakersList,
    down_wakers: WakersList,
    here_wakers: WakersList,
}

pub(crate) struct ListenerGroup<'a> {
    wakers: RefMut<'a, WakersList>,
    version: &'a Cell<u64>,
}
impl<'a> ListenerGroup<'a> {
    pub fn get_version(&mut self) -> u64 {
        self.version.get()
    }
    pub fn increment_version(&mut self) {
        self.version.set(self.version.get() + 1);
        self.wakers.iter().for_each(Waker::wake_by_ref)
    }
    pub fn wakers(&mut self) -> &mut WakersList {
        &mut self.wakers
    }
}

impl Listener {
    pub fn new() -> Self {
        let inner = RefCell::new(ListenerWakers {
            down_wakers: WakersList::new(),
            up_wakers: WakersList::new(),
            here_wakers: WakersList::new(),
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
