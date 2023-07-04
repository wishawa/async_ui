use std::{
    cell::{Cell, RefCell},
    task::Waker,
};

use async_ui_internal_utils::wakers_list::{WakersList, WakersSublist};

/// Listeners associated with an edge.
pub struct Listener<'u> {
    pub(crate) full_list: &'u RefCell<WakersList>,
    pub(crate) up_here_down: [ListenerGroup; 3],
}

pub(crate) struct ListenerGroup {
    pub version: Cell<u64>,
    pub list: WakersSublist,
}
impl ListenerGroup {
    pub fn get_version(&self) -> u64 {
        self.version.get()
    }
    pub fn increment_version(&self, full_list: &WakersList) {
        self.version.set(self.version.get() + 1);
        full_list.iter(&self.list).for_each(Waker::wake_by_ref)
    }
}

impl<'u> Listener<'u> {
    pub fn new(full_list: &'u RefCell<WakersList>) -> Self {
        let mut full_list_borrow = full_list.borrow_mut();
        Self {
            full_list,
            up_here_down: std::array::from_fn(|_| ListenerGroup {
                version: Cell::new(1),
                list: full_list_borrow.add_sublist(),
            }),
        }
    }
    // up version (invalidate downward) = 0
    // here = 1
    // down version (invalidate upward) = 2
    pub fn invalidate<const INDEX: usize>(&self) {
        self.up_here_down[INDEX].increment_version(&mut *self.full_list.borrow_mut());
    }
}
