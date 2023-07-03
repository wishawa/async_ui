use std::{borrow::Borrow, cell::RefCell, collections::HashSet, future::Future, hash::Hash};

use crate::DynamicList;

/**
An easy to use list.

This is less efficient than [DynamicList][crate::DynamicList], but for use cases
with ~1000 items it should be fine.

```
# use async_ui_web::prelude_traits::*;
# use async_ui_web::components::Button;
# use async_ui_web::join;
# use async_ui_web::DiffedList;
# let _ = async {
let list = DiffedList::new(|key: &i32| key.to_string().render());
let mut fibo = vec![1, 1];
let btn = Button::new();
join((
    list.render(),
    btn.render("compute the next fibonacci number".render()),
    async {
        loop {
            list.update(fibo.clone());
            btn.until_click().await;
            fibo.push(fibo.iter().rev().take(2).cloned().sum());
        }
    }
)).await;
# };
```

This list work on *keys*. You give it a Vec of keys (in [update][Self::update]),
along with a way to convert each key into a Future (in [the constructor][Self::new]).
The list will render those Futures in the right order.

When you call [update][Self::update] again, the list will move existing Futures
around so they display in the right order. It will also create new Futures or drop
existing one according to what is present in the keys Vec you supply.
*/
pub struct DiffedList<'c, K, R, F>
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

impl<'c, K, R, F> DiffedList<'c, K, R, F>
where
    K: Eq + Hash + Clone,
    R: FnMut(&K) -> F,
    F: Future + 'c,
{
    /// Create a new DiffedList.
    ///
    /// The argument must be a function/closure that, given a key, returns a
    /// future to render something corresponding to that key.
    pub fn new(renderer: R) -> Self {
        Self {
            dl: DynamicList::new(),
            inner: RefCell::new(Inner {
                prev_keys_list: vec![],
                renderer,
            }),
        }
    }
    /// Display the list here.
    pub async fn render(&self) {
        self.dl.render().await;
    }
    /// Update the list; reorder, insert, or delete futures as needed.
    ///
    /// Time complexity: O(n) in Rust code.
    pub fn update(&self, new_keys: Vec<K>) {
        let mut inner = self.inner.borrow_mut();
        let Inner {
            prev_keys_list,
            renderer,
        } = &mut *inner;
        let mut prev_keys_set = prev_keys_list.iter().collect::<HashSet<_>>();
        let mut prev_keys_iter = prev_keys_list.iter().peekable();
        for key in new_keys.iter() {
            let key = key.borrow();
            if prev_keys_set.remove(key) {
                if prev_keys_iter.next_if_eq(&key).is_some() {
                    continue;
                } else {
                    self.dl.move_before(key, prev_keys_iter.peek().cloned());
                }
            } else {
                let fut = renderer(key);
                self.dl
                    .insert(key.clone(), fut, prev_keys_iter.peek().cloned());
            }
        }
        for key in prev_keys_set.drain() {
            self.dl.remove(key);
        }
        inner.prev_keys_list = new_keys;
    }
    /// Check whether or not the given key is in the list.
    ///
    /// Time complexity: O(1) in Rust/JS code.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.dl.contains_key(key)
    }
}
