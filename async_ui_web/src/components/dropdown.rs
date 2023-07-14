use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    future::Future,
    hash::Hash,
    ops::Deref,
};

use async_ui_web_html::{
    events::EmitHtmlElementEvent,
    nodes::{Option as OptElem, Select},
};
use futures_lite::Stream;
use wasm_bindgen::UnwrapThrowExt;

use crate::NoChild;

/// A more predictable `<select>`.
///
/// You can always implement your own dropdown menu in Async UI Web with
/// [Select][crate::html::Select] and [Option][crate::html::Option].
///
/// The Dropdown implementation provided here wraps `<select>` in a convenient
/// API. You can use `Eq + Hash + Clone` Rust types as options, and easily
/// update them.
///
/// ```
/// # use async_ui_web::{components::Dropdown, prelude_traits::*, join};
/// # let _ = async {
/// let dropdown = Dropdown::<i32>::new();
///
/// // set the options to display
/// dropdown.update_options([
///     (1, "First"),
///     (2, "Second"),
///     (3, "Third")
/// ]);
///
/// join((
///     dropdown.render(),
///     async {
///         loop {
///             dropdown.until_change().await;
///             let value: Option<i32> = dropdown.value();
///         }
///     }
/// )).await;
/// # };
/// ```
///
/// This type [Deref]s to [Select] and [HtmlSelectElement][web_sys::HtmlSelectElement],
/// so you can use all the HTML methods (such as [set_disabled][web_sys::HtmlSelectElement::set_disabled]) on it.
pub struct Dropdown<O: Eq + Hash + Clone> {
    select: Select,
    inner: RefCell<Inner<O>>,
}

impl<O: Eq + Hash + Clone> Default for Dropdown<O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: Eq + Hash + Clone> Deref for Dropdown<O> {
    type Target = Select;
    fn deref(&self) -> &Self::Target {
        &self.select
    }
}

struct Inner<O> {
    selected: Option<O>,
    prev_opts_list: Vec<O>,
    map: HashMap<O, OptElem>,
}

impl<O: Eq + Hash + Clone> Dropdown<O> {
    /// Create a new, empty Dropdown.
    ///
    /// Use [update_options][Self::update_options] to set the options to show.
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
    /// Check if the given option is in the menu.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        O: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.inner.borrow().map.contains_key(key)
    }
    /// Set the selected option in the dropdown.
    ///
    /// If the given value is `None` or is not a valid option,
    /// then deselect the current selection.
    pub fn set_value(&self, opt: Option<O>) {
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
    /// Set the options to be displayed in the dropdown.
    ///
    /// Argument must be an [IntoIterator] yielding a tuple with the option type
    /// and an [str]. The option type will identify that option (e.g. it is what
    /// you will get from [value()][Self::value] if the user selects it) and the
    /// `str` will be the displayed text.
    pub fn update_options<'t>(&'t self, new_opts: impl IntoIterator<Item = (O, &'t str)>) {
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
    /// Get a [Stream] that fires every time the selection is changed by the user.
    ///
    /// To be precise, this fires every time there is a [change event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/change_event)
    /// on the `<select>` tag.
    pub fn until_change(
        &self,
    ) -> impl Future<Output = web_sys::Event> + Stream<Item = web_sys::Event> + '_ {
        self.select.until_change()
    }
    /// Get the currently selected option.
    ///
    /// Returns `None` if nothing is currently selected.
    pub fn value(&self) -> Option<O> {
        let si = self.select.selected_index();
        let inner = self.inner.borrow();
        (si >= 0)
            .then(|| inner.prev_opts_list.get(si as usize).cloned())
            .flatten()
            .or_else(|| inner.selected.clone())
    }
}
