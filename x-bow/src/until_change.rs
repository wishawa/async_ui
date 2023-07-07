use std::{cell::RefCell, task::Poll};

use async_ui_internal_utils::wakers_list::WakerSlot;
use futures_core::Stream;

use crate::{hash_visitor::HashVisitor, path::Path, wakers::StoreWakers, HasherType};

pub struct UntilChange<'a> {
    store: &'a RefCell<StoreWakers>,
    last_version: u64,
    slots: Vec<(u64, WakerSlot)>,
}

impl<'a> UntilChange<'a> {
    pub(crate) fn new<M: Path + ?Sized>(store: &'a RefCell<StoreWakers>, mapper: &M) -> Self {
        Self {
            slots: {
                let mut store = store.borrow_mut();
                let mut slots = Vec::new();
                let mut visitor = HashVisitor {
                    hasher: HasherType::new(),
                    behavior: crate::hash_visitor::HashVisitorBehavior::BuildListeners {
                        wakers: &mut *store,
                        notifiers_list: &mut slots,
                    },
                };
                mapper.visit_hashes(&mut visitor);
                slots
            },
            store,
            last_version: 0,
        }
    }
}

impl<'a> Stream for UntilChange<'a> {
    type Item = ();

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut store = this.store.borrow_mut();
        let new_waker = cx.waker();
        let new_version = this.slots.iter().fold(0, |ver, (hash, slot)| {
            let mut entry = store.get_entry(*hash);
            entry.set_waker(slot, new_waker);
            ver + entry.get_version()
        });
        match std::mem::replace(&mut this.last_version, new_version) {
            lv @ 1.. if lv != new_version => Poll::Ready(Some(())),
            _ => Poll::Pending,
        }
    }
}

impl<'a> Drop for UntilChange<'a> {
    fn drop(&mut self) {
        let mut store = self.store.borrow_mut();
        self.slots.iter().for_each(|(hash, slot)| {
            store.remove_waker_slot(*hash, slot);
        });
    }
}
