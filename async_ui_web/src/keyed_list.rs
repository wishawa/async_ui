use std::{borrow::Borrow, cell::RefCell, collections::HashSet, future::Future, hash::Hash};

use crate::DynamicList;

pub struct KeyedList<'c, K, R, F>
where
    K: Eq + Hash + Clone,
    R: FnMut(&K) -> F,
    F: Future + 'c,
{
    dl: DynamicList<'c, K, F>,
    inner: RefCell<Inner<K, R>>,
}

struct Inner<K, R> {
    prev_keys_list: Vec<K>,
    renderer: R,
}

impl<'c, K, R, F> KeyedList<'c, K, R, F>
where
    K: Eq + Hash + Clone,
    R: FnMut(&K) -> F,
    F: Future + 'c,
{
    pub fn new(renderer: R) -> Self {
        Self {
            dl: DynamicList::new(),
            inner: RefCell::new(Inner {
                prev_keys_list: vec![],
                renderer,
            }),
        }
    }
    pub async fn render(&self) {
        self.dl.render().await;
    }
    pub fn update(&self, keys: impl Iterator<Item = K>) {
        let mut inner = self.inner.borrow_mut();
        let new_keys: Vec<_> = keys.collect();
        let Inner {
            prev_keys_list,
            renderer,
        } = &mut *inner;
        let mut prev_keys_set: HashSet<_> = prev_keys_list.iter().collect();
        let mut prev_keys_iter = prev_keys_list.iter().peekable();
        for key in new_keys.iter() {
            if prev_keys_set.remove(key) {
                if prev_keys_iter.peek() != Some(&key) {
                    self.dl.move_before(key, None);
                } else {
                    prev_keys_iter.next();
                }
            } else {
                self.dl.insert(key.clone(), renderer(key), None);
            }
        }
        for key in prev_keys_set.drain() {
            self.dl.remove(key);
        }
        inner.prev_keys_list = new_keys;
    }
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.dl.contains_key(key)
    }
}
