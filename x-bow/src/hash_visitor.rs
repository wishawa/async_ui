use std::{
    hash::Hasher,
    ops::{Deref, DerefMut},
};

use async_ui_internal_utils::wakers_list::WakerSlot;

use crate::{hash::WakerHashEntry, wakers::StoreWakers};

pub(crate) type HasherType = std::collections::hash_map::DefaultHasher;

/// Used internally to avoid needing to make [visit_hashes][crate::Path::visit_hashes]
/// a generic method.
pub struct HashVisitor<'a> {
    pub(crate) hasher: HasherType,
    pub(crate) behavior: HashVisitorBehavior<'a>,
}

pub(crate) enum HashVisitorBehavior<'a> {
    BuildRegularListeners {
        wakers: &'a mut StoreWakers,
        notifiers_list: &'a mut Vec<(WakerHashEntry, WakerSlot)>,
    },
    GetHash {},
    WakeListeners {
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
        match &mut self.behavior {
            HashVisitorBehavior::BuildRegularListeners {
                wakers,
                notifiers_list,
            } => {
                let hash = WakerHashEntry::regular_from(self.hasher.finish());
                let slot = wakers.get_entry(hash).add_waker_slot();
                notifiers_list.push((hash, slot));
            }
            HashVisitorBehavior::GetHash {} => {}
            HashVisitorBehavior::WakeListeners { wakers } => {
                let hash = WakerHashEntry::bubbling_from(self.hasher.finish());
                wakers.get_entry(hash).wake();
            }
        }
    }
}
