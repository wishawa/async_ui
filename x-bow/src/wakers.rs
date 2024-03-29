use std::task::Waker;

use async_ui_internal_utils::wakers_arena::{WakerSlot, WakersArena, WakersSublist};

use nohash_hasher::IntMap;

use crate::hash::WakerHashEntry;

pub struct StoreWakers {
    all_wakers: WakersArena,
    map: IntMap<u64, Entry>,
}

struct Entry {
    sublist: WakersSublist,
    version: u64,
}
impl Entry {
    fn new(all_wakers: &mut WakersArena) -> Self {
        Self {
            sublist: all_wakers.add_sublist(),
            version: 1,
        }
    }
}

pub(crate) struct WakerApi<'a> {
    entry: &'a mut Entry,
    all: &'a mut WakersArena,
}
impl<'a> WakerApi<'a> {
    pub fn add_waker_slot(&mut self) -> WakerSlot {
        self.all.add(&self.entry.sublist)
    }
    pub fn set_waker(&mut self, slot: &WakerSlot, waker: &Waker) {
        self.all.update(slot, waker)
    }
    pub fn get_version(&mut self) -> u64 {
        self.entry.version
    }
}

impl StoreWakers {
    pub(crate) fn new() -> Self {
        Self {
            all_wakers: WakersArena::new(),
            map: IntMap::default(),
        }
    }
    pub(crate) fn get_entry(&mut self, hash: WakerHashEntry) -> WakerApi<'_> {
        WakerApi {
            entry: self
                .map
                .entry(hash.value())
                .or_insert_with(|| Entry::new(&mut self.all_wakers)),
            all: &mut self.all_wakers,
        }
    }
    pub(crate) fn wake_entry(&mut self, hash: WakerHashEntry) {
        if let Some(entry) = self.map.get_mut(&hash.value()) {
            entry.version += 1;
            self.all_wakers
                .iter(&entry.sublist)
                .for_each(Waker::wake_by_ref);
        }
    }
    pub(crate) fn remove_waker_slot(&mut self, hash: WakerHashEntry, slot: &WakerSlot) {
        match self.map.entry(hash.value()) {
            std::collections::hash_map::Entry::Occupied(occ) => {
                self.all_wakers.remove(slot);
                let sublist = &occ.get().sublist;
                if self.all_wakers.remove_sublist_if_empty(sublist) {
                    occ.remove();
                }
            }
            std::collections::hash_map::Entry::Vacant(_) => {
                // remove nonexistant waker
                panic!()
            }
        }
    }
}
