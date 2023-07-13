use std::{
    cell::Cell,
    collections::{HashSet, VecDeque},
    future::Future,
    hash::Hash,
    ops::Deref,
    rc::Rc,
};

use crate::DynamicList;

type Version = u64;

/// Data for [ModeledList].
///
/// This struct wraps a [Vec] and tracks changes so that the displayed list
/// can be updated efficiently. It exposes a subset of Vec's API.
///
/// ListModel does not provide any form of reactivity in itself.
/// Use it with some reactive wrapper or signaling system.
#[derive(Clone, Debug)]
pub struct ListModel<K> {
    vec: Vec<K>,
    log: ListModelLog<K>,
}

impl<K> Default for ListModel<K> {
    fn default() -> Self {
        Self {
            vec: Default::default(),
            log: Default::default(),
        }
    }
}

#[derive(Debug)]
struct ListModelLog<K> {
    changes: VecDeque<Change<K>>,
    version: Version,
    // A unique Rc so that ModeledList knows that `update` can be applied
    // based on the log.
    unique: Rc<()>,
}
impl<K> Clone for ListModelLog<K> {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl<K> Default for ListModelLog<K> {
    fn default() -> Self {
        Self {
            changes: VecDeque::new(),
            version: 0,
            unique: Rc::new(()),
        }
    }
}

#[derive(Debug)]
enum Change<K> {
    Insert { key: K, before: Option<K> },
    Remove { key: K },
    Swap { a: K, b: K },
    Move { to_move: K, before: Option<K> },
}

impl<K: Clone> ListModel<K> {
    /// Create a new, empty `ListModel<K>`.
    ///
    /// It is prefereable to use the `From<Vec<K>>` impl instead if you
    /// already have data.
    pub fn new() -> Self {
        Self::default()
    }
    fn add_change(&mut self, change: Change<K>) {
        self.log.changes.push_back(change);
        self.log.version += 1;
        let max_log_len = self.vec.len() / 2;
        let log_len = self.log.changes.len();
        if log_len > max_log_len {
            self.log.changes.pop_front();
        }
    }
    /// Like [Vec::insert].
    pub fn insert(&mut self, index: usize, key: K) {
        self.add_change(Change::Insert {
            key: key.clone(),
            before: self.vec.get(index).cloned(),
        });
        self.vec.insert(index, key);
    }
    /// Like [Vec::remove].
    pub fn remove(&mut self, index: usize) -> K {
        let removed = self.vec.remove(index);
        self.add_change(Change::Remove {
            key: removed.clone(),
        });
        removed
    }
    /// Like [Vec::pop].
    pub fn pop(&mut self) -> Option<K> {
        let res = self.vec.pop();
        if let Some(key) = res.clone() {
            self.add_change(Change::Remove { key });
        }
        res
    }
    /// Like [Vec::push].
    pub fn push(&mut self, key: K) {
        self.add_change(Change::Insert {
            key: key.clone(),
            before: None,
        });
        self.vec.push(key);
    }
    /// Like [slice::swap].
    pub fn swap(&mut self, a: usize, b: usize) {
        self.add_change(Change::Swap {
            a: self.vec[a].clone(),
            b: self.vec[b].clone(),
        });
        self.vec.swap(a, b);
    }
    /// Move the item at index `from` to be at index `to`.
    /// **Panics** if either index is out of bound.
    pub fn move_item(&mut self, from: usize, to: usize) {
        let low = from.min(to);
        let high = from.max(to);
        let slice = &mut self.vec[low..=high];
        if from < to {
            slice.rotate_left(1);
        } else if to < from {
            slice.rotate_right(1);
        }
        self.add_change(Change::Move {
            to_move: self.vec[to].clone(),
            before: self.vec.get(to + 1).cloned(),
        })
    }
    /// Get mutable access to the underlying Vec.
    ///
    /// This breaks change tracking;
    /// on the next call, [ModeledList::update] will have to go through
    /// all the elements again.
    pub fn modify_vec(&mut self) -> &mut Vec<K> {
        self.log.version += 1;
        self.log.changes.clear();
        &mut self.vec
    }
}

impl<K> Deref for ListModel<K> {
    type Target = [K];

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<K> From<Vec<K>> for ListModel<K> {
    fn from(vec: Vec<K>) -> Self {
        Self {
            vec,
            log: Default::default(),
        }
    }
}
impl<K> FromIterator<K> for ListModel<K> {
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}

/// For rendering an array-like dynamic collection of items.
///
/// Similar to [DiffedList][super::DiffedList], but more efficient.
/// While DiffedList works on [Vec] data, [ModeledList] works on
/// [ListModel] (which provide a subset of [Vec]'s API).
///
/// To use [ModeledList], wrap your data in [ListModel], modify the model
/// as needed, and call [update][ModeledList::update] after modifications.
///
/// ```
/// # use async_ui_web::prelude_traits::*;
/// # use async_ui_web::components::Button;
/// # use async_ui_web::join;
/// # use async_ui_web::{ModeledList, ListModel};
/// # let _ = async {
/// let list = ModeledList::new(|key: &i32| key.to_string().render());
/// let mut fibo = ListModel::from(vec![1, 1]);
/// let btn = Button::new();
/// join((
///     list.render(),
///     btn.render("compute the next fibonacci number".render()),
///     async {
///         loop {
///             // sync the UI to the model
///             list.update(&fibo);
///             btn.until_click().await;
///
///             // update the model
///             fibo.push(fibo.iter().rev().take(2).cloned().sum());
///         }
///     }
/// )).await;
/// # };
/// ```
pub struct ModeledList<'c, K: Eq + Hash, F: Future, R: Fn(&K) -> F> {
    list: DynamicList<'c, K, F>,
    version: Cell<Version>,
    renderer: R,
    unique: Cell<Option<Rc<()>>>,
}

impl<'c, K: Eq + Hash + Clone, F: Future, R: Fn(&K) -> F> ModeledList<'c, K, F, R> {
    /// Create a new ModeledList.
    ///
    /// The argument must be a function/closure that, given a key, returns a
    /// future to render something corresponding to that key.
    pub fn new(renderer: R) -> Self {
        Self {
            list: DynamicList::new(),
            version: Cell::new(0),
            renderer,
            unique: Cell::new(None),
        }
    }
    pub async fn render(&self) {
        self.list.render().await;
    }
    /// Update the list to reflect the model.
    ///
    /// This method will:
    /// *   drop rendered futures corresponding to keys that are not in the model
    /// *   call the `renderer` closure given to [new][Self::new] for each key
    ///     that has been added to the model since the last `update` call
    /// *   move the futures so that things appear in the same order as the
    ///     keys in the model
    ///
    /// Time complexity: O(C)
    /// where C is the number of changes done to the model since last `update` call.
    ///
    /// Repeated calls of `update` should pass same instance of `ListModel`.
    /// If not, performance will suffer (although the list will still work fine).
    pub fn update(&self, model: &ListModel<K>) {
        let new_ver = model.log.version;
        let unique = self.unique.take();
        let old_ver = if unique.is_some_and(|rc| Rc::ptr_eq(&rc, &model.log.unique)) {
            // same instance of `ListModel` as last time
            Some(self.version.replace(new_ver))
        } else {
            // a different `ListModel` instance
            self.unique.set(Some(model.log.unique.clone()));
            None
        };
        let log_start_ver = new_ver - model.log.changes.len() as u64;
        match old_ver {
            Some(old_ver) if old_ver >= log_start_ver => {
                // we can update using the log
                model
                    .log
                    .changes
                    .iter()
                    .skip((old_ver - log_start_ver) as usize)
                    .for_each(|ch| match ch {
                        Change::Insert { key, before } => {
                            assert!(!self.list.insert(
                                key.clone(),
                                (self.renderer)(key),
                                before.as_ref()
                            ),);
                        }
                        Change::Remove { key } => {
                            assert!(self.list.remove(key));
                        }
                        Change::Swap { a, b } => {
                            self.list.swap(a, b);
                        }
                        Change::Move { to_move, before } => {
                            self.list.move_before(to_move, before.as_ref());
                        }
                    });
            }
            _ => {
                // too outdated, rebuild the list
                let mut new_keys = model.vec.iter().collect::<HashSet<_>>();
                let mut retained_keys = HashSet::new();
                // remove all the keys that are no longer in the list
                // build a set of retained keys
                self.list.retain(|k| {
                    if let Some(key) = new_keys.take(k) {
                        retained_keys.insert(key);
                        true
                    } else {
                        false
                    }
                });
                // insert all the new keys
                model.vec.iter().for_each(|key| {
                    if !retained_keys.contains(key) {
                        self.list.insert(key.clone(), (self.renderer)(key), None);
                    }
                });
                // move every retained key to the right location
                model.vec.iter().rev().fold(None, |last, key| {
                    if retained_keys.contains(key) {
                        self.list.move_before(key, last);
                    }
                    Some(key)
                });
            }
        }
    }
}
