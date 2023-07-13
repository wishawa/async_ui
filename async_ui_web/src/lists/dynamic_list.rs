use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    future::{pending, Future},
    hash::Hash,
    pin::Pin,
};

use async_executor::{LocalExecutor, Task};
use async_ui_web_core::{ContainerNodeFuture, DetachmentBlocker, SiblingNodeFuture};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{Comment, DocumentFragment};

#[derive(Clone, Debug)]
enum ContainingNode {
    Real(web_sys::Node),
    Fake(DocumentFragment),
}

/**
For rendering many futures, and adding/removing them dynamically.

Futures (and the stuff they render) can be
inserted, removed, or reordered within the list.

Only use this if you need low level control.
Often, [ModeledList][super::ModeledList] is easier to use.

```rust
# use async_ui_web::lists::DynamicList;
# use async_ui_web::html::Button;
# use async_ui_web::join;
# use async_ui_web::prelude_traits::*;
# let _ = async {
let list = DynamicList::new();
let add_item = Button::new();
let mut item_key_counter = 0;
join((
    list.render(),
    add_item.render("add an item".render()),
    async {
        loop {
            add_item.until_click().await;
            list.insert(item_key_counter, "another item".render(), None);
            item_key_counter += 1;
        }
    }
)).await;
# };
```
*/
pub struct DynamicList<'c, K: Eq + Hash, F: Future + 'c> {
    inner: RefCell<DynamicListInner<K, F>>,
    executor: LocalExecutor<'c>,
    list_end_marker: web_sys::Node,
    list_start_marker: web_sys::Node,
    detachment_blocker: DetachmentBlocker,
}

struct DynamicListInner<K: Eq + Hash, F: Future> {
    items: HashMap<K, Stored<F>>,
    containing_node: ContainingNode,
}

struct Stored<F: Future> {
    task: Task<F::Output>,
    start_marker: web_sys::Node,
    end_marker: web_sys::Node,
}

impl ContainingNode {
    fn get(&self) -> &web_sys::Node {
        match self {
            ContainingNode::Real(real) => real,
            ContainingNode::Fake(fake) => fake,
        }
    }
}

impl<'c, K: Eq + Hash, F: Future + 'c> Default for DynamicList<'c, K, F> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'c, K: Eq + Hash, F: Future + 'c> DynamicList<'c, K, F> {
    /// Create a new list, without anything in it.
    pub fn new() -> Self {
        let frag = DocumentFragment::new().unwrap_throw();
        let list_end_marker = Comment::new().unwrap_throw().into();
        let list_start_marker = Comment::new().unwrap_throw().into();
        frag.append_child(&list_start_marker).unwrap_throw();
        frag.append_child(&list_end_marker).unwrap_throw();
        Self {
            inner: RefCell::new(DynamicListInner {
                containing_node: ContainingNode::Fake(frag),
                items: HashMap::new(),
            }),
            executor: LocalExecutor::new(),
            list_end_marker,
            list_start_marker,
            detachment_blocker: DetachmentBlocker,
        }
    }
    /// Insert a future to render in the list.
    ///
    /// You supply a key along with the future to render.
    /// This key can later be used to delete or move this future.
    ///
    /// If that key is already taken, the new future replaces the old one and
    /// the method returns true.
    ///
    /// The last argument (`before`) specifies where the future should be added.
    /// This works similarly to [insertBefore](https://developer.mozilla.org/en-US/docs/Web/API/Node/insertBefore);
    /// you specify the key of the node *before* which your new future should
    /// appear, or None if you want to insert at the end.
    ///
    /// If the given `before` key doesn't exist, the future is inserted at the end.
    ///
    /// Time complexity: O(1) in Rust/JS code.
    pub fn insert(&self, key: K, future: F, before: Option<&K>) -> bool {
        let mut inner = self.inner.borrow_mut();
        let container = inner.containing_node.get();
        let start_marker: web_sys::Node = Comment::new().unwrap_throw().into();
        let end_marker: web_sys::Node = Comment::new().unwrap_throw().into();
        let after = before
            .map(|k| &inner.items.get(k).unwrap().start_marker)
            .unwrap_or(&self.list_end_marker);
        container
            .insert_before(&end_marker, Some(after))
            .unwrap_throw();
        container
            .insert_before(&start_marker, Some(&end_marker))
            .unwrap_throw();
        let end_marker_cloned = end_marker.clone();
        let task = self
            .executor
            .spawn(SiblingNodeFuture::new(future, end_marker_cloned));
        let stored = Stored {
            task,
            start_marker,
            end_marker,
        };
        if let Some(Stored {
            task,
            start_marker,
            end_marker,
        }) = inner.items.insert(key, stored)
        {
            drop(task);
            let container = inner.containing_node.get();
            let _ = container.remove_child(&start_marker).unwrap_throw();
            let _ = container.remove_child(&end_marker).unwrap_throw();
            true
        } else {
            false
        }
    }
    /// Remove the future inserted with the specified key.
    /// Returns whether or not the future at that key was in the list
    /// (i.e. returns true iff something was removed).
    ///
    /// Time complexity: O(1) in Rust/JS code.
    pub fn remove<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut inner = self.inner.borrow_mut();
        if let Some(Stored {
            start_marker,
            end_marker,
            task,
        }) = inner.items.remove(key)
        {
            drop(task);
            let container = inner.containing_node.get();
            let _ = container.remove_child(&start_marker).unwrap_throw();
            let _ = container.remove_child(&end_marker).unwrap_throw();
            true
        } else {
            false
        }
    }
    /// Check if there is a future associated with the given key.
    /// Returns true if there is one.
    ///
    /// Time complexity: O(1) in Rust/JS code.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.inner.borrow().items.contains_key(key)
    }
    /// Move the future at the key given in the first argument (`to_move`) so that
    /// it appears just before the future at the key given in the second argument (`before`).
    ///
    /// **Panics** if the key in the first argument (`to_move`) is not in the list.
    ///
    /// If the key in the second argument (`before`) is None or doesn't exist,
    /// the future moved to the end of the list.
    ///
    /// Time complexity: O(number of HTML nodes moved) in Rust/JS code.
    /// Unless you're doing something weird like rendering a list as a direct child of a list,
    /// the number of HTML nodes will likely be O(1).
    pub fn move_before<Q>(&self, to_move: &Q, before: Option<&Q>)
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let inner = self.inner.borrow();
        let after = before
            .map(|k| &inner.items.get(k).unwrap().start_marker)
            .unwrap_or(&self.list_end_marker);
        let to_move = inner.items.get(to_move).unwrap();
        let container = inner.containing_node.get();
        move_nodes_before(
            container,
            &to_move.start_marker,
            &to_move.end_marker,
            Some(after),
        );
    }
    /// Swap the position of two futures.
    ///
    /// **Panics** if either of the keys don't exist in the list.
    ///
    /// Time complexity: O(number of HTML nodes moved) in Rust/JS code.
    /// Unless you're doing something weird like rendering a list as a direct child of a list,
    /// the number of HTML nodes will likely be O(1).
    pub fn swap<Q>(&self, key_1: &Q, key_2: &Q)
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let inner = self.inner.borrow();
        let item_1 = inner.items.get(key_1).unwrap();
        let item_2 = inner.items.get(key_2).unwrap();
        if std::ptr::eq(item_1, item_2) {
            return;
        }
        let after_key_1 = item_1.end_marker.next_sibling();
        let container = inner.containing_node.get();
        move_nodes_before(
            container,
            &item_1.start_marker,
            &item_1.end_marker,
            Some(&item_2.start_marker),
        );
        move_nodes_before(
            container,
            &item_2.start_marker,
            &item_2.end_marker,
            after_key_1.as_ref(),
        );
    }
    #[doc(hidden)]
    pub fn order<'t>(&self, _keys: impl IntoIterator<Item = &'t K>)
    where
        K: 't,
    {
        todo!()
    }
    /// Retain only the items satisfying the predicate.
    ///
    /// Items for which `func` returns `false` will be removed.
    pub fn retain(&self, mut func: impl FnMut(&K) -> bool) {
        let inner = &mut *self.inner.borrow_mut();
        let DynamicListInner {
            items,
            containing_node,
        } = inner;
        let container = containing_node.get();
        items.retain(|key, value| {
            let keep = func(key);
            if !keep {
                container.remove_child(&value.start_marker).unwrap_throw();
                container.remove_child(&value.end_marker).unwrap_throw();
            }
            keep
        });
    }
    /// Render the list here.
    ///
    /// This future never completes.
    /// Race it with some other future if you want to drop it eventually.
    pub async fn render(&self) {
        let real_containing_node;
        {
            use async_ui_internal_utils::dummy_waker::dummy_waker;
            use std::task::Context;
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            let mut insert_marker_fut =
                ContainerNodeFuture::new(pending::<()>(), self.list_start_marker.clone());
            let _ = Pin::new(&mut insert_marker_fut).poll(&mut context);
            real_containing_node = self.list_start_marker.parent_node().unwrap_throw();
            real_containing_node
                .insert_before(&self.list_end_marker, Some(&self.list_start_marker))
                .unwrap_throw();
            drop(insert_marker_fut);
            real_containing_node
                .insert_before(&self.list_start_marker, Some(&self.list_end_marker))
                .unwrap_throw();
        }
        let stored_fragment;
        {
            let mut inner = self.inner.borrow_mut();
            match std::mem::replace(
                &mut inner.containing_node,
                ContainingNode::Real(real_containing_node.clone()),
            ) {
                ContainingNode::Real(_) => panic!("rendering in more than one places not allowed"),
                ContainingNode::Fake(fragment) => {
                    real_containing_node
                        .insert_before(fragment.as_ref(), Some(&self.list_end_marker))
                        .unwrap_throw();
                    stored_fragment = fragment;
                }
            }
        }
        let _guard = scopeguard::guard((), |_| {
            let mut inner = self.inner.borrow_mut();
            let fragment = stored_fragment.clone();
            move_nodes_before(
                fragment.as_ref(),
                &self.list_start_marker,
                &self.list_end_marker,
                None,
            );
            inner.containing_node = ContainingNode::Fake(fragment);
        });
        self.executor.run(pending()).await
    }
}

impl<'c, K: Eq + Hash, F: Future> Drop for DynamicList<'c, K, F> {
    fn drop(&mut self) {
        self.detachment_blocker.block_until_drop();
    }
}
/// Move `start_marker`, `end_marker`, and eveything between them
/// into `container` at location before `after`.
fn move_nodes_before(
    container: &web_sys::Node,
    start_marker: &web_sys::Node,
    end_marker: &web_sys::Node,
    after: Option<&web_sys::Node>,
) {
    let mut node = start_marker.clone();
    loop {
        let next_node = node.next_sibling();
        container.insert_before(&node, after).unwrap_throw();
        if end_marker.is_same_node(Some(&node)) {
            break;
        }
        node = next_node.unwrap();
    }
}
