use std::{
    hash::Hasher,
    ops::{Deref, DerefMut},
};

use async_ui_internal_utils::wakers_arena::WakerSlot;

use crate::{hash::WakerHashEntry, wakers::StoreWakers};

pub(crate) type HasherType = std::collections::hash_map::DefaultHasher;

/// Used internally to avoid needing to make [visit_hashes][crate::Path::visit_hashes]
/// a generic method.
pub struct HashVisitor<'a> {
    hasher: HasherType,
    behavior: HashVisitorBehavior<'a>,
    depth: u8,
}

impl<'a> HashVisitor<'a> {
    pub(crate) fn new(behavior: HashVisitorBehavior<'a>) -> Self {
        Self {
            hasher: HasherType::new(),
            behavior,
            depth: 0,
        }
    }
    pub(crate) fn to_regular_key(&self) -> WakerHashEntry {
        WakerHashEntry::regular_from(self.hasher.finish(), self.depth)
    }
    pub(crate) fn to_bubbling_key(&self) -> WakerHashEntry {
        WakerHashEntry::bubbling_from(self.hasher.finish())
    }
}

pub(crate) enum HashVisitorBehavior<'a> {
    BuildRegularListeners {
        wakers: &'a mut StoreWakers,
        notifiers_list: &'a mut Vec<(WakerHashEntry, WakerSlot)>,
    },
    GetHash {},
    WakeBubblingListeners {
        wakers: &'a mut StoreWakers,
    },
}

impl<'a> Deref for HashVisitor<'a> {
    type Target = HasherType;

    fn deref(&self) -> &Self::Target {
        &self.hasher
    }
}

impl<'a> DerefMut for HashVisitor<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hasher
    }
}

impl<'a> HashVisitor<'a> {
    pub fn finish_one(&mut self) {
        self.depth = self.depth.wrapping_add(1);
        match &mut self.behavior {
            HashVisitorBehavior::BuildRegularListeners {
                wakers,
                notifiers_list,
            } => {
                let hash = WakerHashEntry::regular_from(self.hasher.finish(), self.depth);
                let slot = wakers.get_entry(hash).add_waker_slot();
                notifiers_list.push((hash, slot));
            }
            HashVisitorBehavior::GetHash {} => {}
            HashVisitorBehavior::WakeBubblingListeners { wakers } => {
                let hash = WakerHashEntry::bubbling_from(self.hasher.finish());
                wakers.wake_entry(hash);
            }
        }
    }
}
