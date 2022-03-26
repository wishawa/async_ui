use std::task::Waker;

use slab::Slab;

pub(crate) struct SubscriptionKey(usize);

pub(crate) struct Subscriptions {
    wakers: Slab<Waker>,
}
impl Subscriptions {
    pub fn new() -> Self {
        Self {
            wakers: Slab::with_capacity(4),
        }
    }
    pub fn add(&mut self, waker: Waker) -> SubscriptionKey {
        let key = self.wakers.insert(waker);
        SubscriptionKey(key)
    }
    pub fn wake_all(&self) {
        for (_key, waker) in self.wakers.iter() {
            waker.wake_by_ref();
        }
    }
    pub fn remove(&mut self, key: SubscriptionKey) {
        self.wakers.remove(key.0);
    }
}
