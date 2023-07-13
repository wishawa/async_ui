use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    future::Future,
    hash::Hash,
    ops::Deref,
};

use async_ui_web_components::{
    components::{Option as OptElem, Select},
    events::EmitEditEvent,
};
use futures_lite::Stream;
use wasm_bindgen::UnwrapThrowExt;

use crate::NoChild;

pub struct Dropdown<K: Eq + Hash + Clone> {
    select: Select,
    inner: RefCell<Inner<K>>,
}

impl<K: Eq + Hash + Clone> Default for Dropdown<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + Clone> Deref for Dropdown<K> {
    type Target = Select;
    fn deref(&self) -> &Self::Target {
        &self.select
    }
}

struct Inner<K> {
    selected: Option<K>,
    prev_opts_list: Vec<K>,
    map: HashMap<K, OptElem>,
}

impl<K: Eq + Hash + Clone> Dropdown<K> {
    pub fn new() -> Self {
        Self {
            select: Select::new(),
            inner: RefCell::new(Inner {
                selected: None,
                prev_opts_list: vec![],
                map: HashMap::new(),
            }),
        }
    }
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.inner.borrow().map.contains_key(key)
    }
    pub fn set_value(&self, opt: Option<K>) {
        if let Some(opt_ref) = opt.as_ref() {
            let inner = self.inner.borrow();
            if let Some(elem) = inner.map.get(opt_ref) {
                elem.set_selected(true);
                drop(inner);
                self.inner.borrow_mut().selected = opt;
                return;
            }
        }
        self.select.set_value("");
    }
    pub fn set_options<'t>(&'t self, new_opts: impl IntoIterator<Item = (K, &'t str)>) {
        let current_value = self.value();
        let mut bm = self.inner.borrow_mut();
        let Inner {
            selected,
            prev_opts_list,
            map,
        } = &mut *bm;
        *selected = current_value;

        let mut prev_opts_set = prev_opts_list.iter().collect::<HashSet<_>>();
        let mut prev_opts_iter = prev_opts_list.iter().peekable();
        let mut new_opts_list = Vec::new();
        for (opt, text) in new_opts {
            let should_put = if prev_opts_set.remove(&opt) {
                prev_opts_iter.next_if_eq(&&opt).is_none()
            } else {
                let elem = OptElem::new();
                map.insert(opt.clone(), elem);
                true
            };
            if should_put {
                self.select
                    .insert_before(
                        map.get(&opt).unwrap(),
                        prev_opts_iter
                            .peek()
                            .map(|next| map.get(next).unwrap().as_ref()),
                    )
                    .unwrap_throw();
            }
            map.get(&opt).unwrap().set_text_content(Some(text));
            new_opts_list.push(opt);
        }

        for opt in prev_opts_set.drain() {
            self.select
                .remove_child(map.remove(opt).unwrap().as_ref())
                .unwrap_throw();
        }
        *prev_opts_list = new_opts_list;
        if let Some(sel) = selected.as_ref().and_then(|k| map.get(k)) {
            sel.set_selected(true);
        } else {
            self.select.set_value("");
        }
    }
    pub async fn render(&self) {
        self.select.render(NoChild).await;
    }
    pub fn until_change(
        &self,
    ) -> impl Future<Output = web_sys::Event> + Stream<Item = web_sys::Event> + '_ {
        self.select.until_change()
    }
    pub fn value(&self) -> Option<K> {
        let si = self.select.selected_index();
        let inner = self.inner.borrow();
        (si >= 0)
            .then(|| inner.prev_opts_list.get(si as usize).cloned())
            .flatten()
            .or_else(|| inner.selected.clone())
    }
}
