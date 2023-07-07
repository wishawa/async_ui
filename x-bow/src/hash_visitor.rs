use std::{
    hash::Hasher,
    ops::{Deref, DerefMut},
};

use async_ui_internal_utils::wakers_list::WakerSlot;

use crate::{wakers::StoreWakers, HasherType};

pub struct HashVisitor<'a> {
    pub(crate) hasher: HasherType,
    pub(crate) behavior: HashVisitorBehavior<'a>,
}

pub(crate) enum HashVisitorBehavior<'a> {
    BuildListeners {
        wakers: &'a mut StoreWakers,
        notifiers_list: &'a mut Vec<(u64, WakerSlot)>,
    },
    BuildNotifier {},
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
            HashVisitorBehavior::BuildListeners {
                wakers,
                notifiers_list,
            } => {
                let hash = self.hasher.finish();
                let slot = wakers.get_entry(hash).add_waker_slot();
                notifiers_list.push((hash, slot));
            }
            HashVisitorBehavior::BuildNotifier {} => {}
        }
    }
}
