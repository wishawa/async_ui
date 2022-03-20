use std::{collections::HashMap, ops::AddAssign, task::Waker};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct SubscriptionKey(usize);
struct Subscriptions {
    last: SubscriptionKey,
    map: HashMap<SubscriptionKey, Waker>,
}
impl Subscriptions {
    fn new() -> Self {
        Self {
            last: SubscriptionKey(0),
            map: HashMap::new(),
        }
    }
    fn add(&mut self, waker: Waker) -> SubscriptionKey {
        let last_cpy = self.last.clone();
        if self.map.insert(last_cpy.clone(), waker).is_some() {
            panic!("subscription key already exists")
        }
        self.last.0.add_assign(1);
        last_cpy
    }
    fn remove(&mut self, key: SubscriptionKey) {
        self.map
            .remove(&key)
            .expect("key not found in subscriptions");
    }
}
impl<T> ReactiveInner<T> {
    pub fn get_version(&self) -> usize {
        self.version
    }
    pub fn notify_modified(&mut self) {
        self.version.add_assign(1);
        for waker in self.subscriptions.map.values() {
            waker.wake_by_ref();
        }
    }
    pub fn deref_value(&self) -> &T {
        &self.value
    }
    pub fn deref_value_mut(&mut self) -> &mut T {
        &mut self.value
    }
    pub fn new(value: T) -> Self {
        Self {
            value,
            version: 0,
            subscriptions: Subscriptions::new(),
        }
    }
    pub fn add_subscription(&mut self, waker: Waker) -> SubscriptionKey {
        self.subscriptions.add(waker)
    }
    pub fn remove_subscription(&mut self, key: SubscriptionKey) {
        self.subscriptions.remove(key)
    }
}
pub(crate) struct ReactiveInner<T> {
    version: usize,
    value: T,
    subscriptions: Subscriptions,
}
