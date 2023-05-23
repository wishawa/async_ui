use std::{
    cell::RefCell,
    collections::HashMap,
    future::{pending, poll_fn, Future},
    hash::Hash,
    pin::pin,
    task::Poll,
};

use async_executor::{LocalExecutor, Task};
use async_ui_web_core::{get_containing_node, ContainerNodeFuture, SiblingNodeFuture};
use js_sys::Object;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{Comment, DocumentFragment};

#[derive(Clone, Debug)]
enum ContainingNode {
    Real(web_sys::Node),
    Fake(DocumentFragment),
}

pub struct DynamicList<'c, K: Eq + Hash, F: Future> {
    inner: RefCell<DynamicListInner<K, F>>,
    executor: LocalExecutor<'c>,
    list_end_marker: web_sys::Node,
    list_start_marker: web_sys::Node,
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

impl<'c, K: Eq + Hash, F: Future> DynamicListInner<K, F> {
    pub fn get_container(&self) -> &web_sys::Node {
        match &self.containing_node {
            ContainingNode::Real(real) => real,
            ContainingNode::Fake(fake) => &*fake,
        }
    }
}
impl<'c, K: Eq + Hash, F: Future + 'c> DynamicList<'c, K, F> {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(DynamicListInner {
                containing_node: ContainingNode::Fake(DocumentFragment::new().unwrap_throw()),
                items: HashMap::new(),
            }),
            executor: LocalExecutor::new(),
            list_end_marker: Comment::new().unwrap_throw().into(),
            list_start_marker: Comment::new().unwrap_throw().into(),
        }
    }
    pub fn insert(&self, key: K, future: F, before: Option<&K>) {
        let mut inner = self.inner.borrow_mut();
        let container = inner.get_container();
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
        inner.items.insert(key, stored);
    }
    pub fn remove(&self, key: &K) {
        let mut inner = self.inner.borrow_mut();
        let Stored {
            start_marker,
            end_marker,
            task,
        } = inner.items.remove(key).unwrap();
        drop(task);
        let container = inner.get_container();
        let _ = container.remove_child(&start_marker).unwrap_throw();
        let _ = container.remove_child(&end_marker).unwrap_throw();
    }
    pub fn move_before(&self, to_move: &K, before: Option<&K>) {
        let inner = self.inner.borrow();
        let after = before
            .map(|k| &inner.items.get(k).unwrap().start_marker)
            .unwrap_or(&self.list_end_marker);
        let to_move = inner.items.get(to_move).unwrap();
        let container = inner.get_container();
        move_nodes_before(
            container,
            &to_move.start_marker,
            &to_move.end_marker,
            Some(after),
        );
    }
    pub fn swap(&self, key_1: &K, key_2: &K) {
        let inner = self.inner.borrow();
        let item_1 = inner.items.get(key_1).unwrap();
        let item_2 = inner.items.get(key_2).unwrap();
        if std::ptr::eq(item_1, item_2) {
            return;
        }
        let after_key_1 = item_1.end_marker.next_sibling();
        let container = inner.get_container();
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
    pub fn order<'k>(&self, keys: impl IntoIterator<Item = &'k K>)
    where
        K: 'k,
    {
        todo!()
    }
    pub async fn render(&self) {
        let real_containing_node = get_containing_node();
        let mut list_end_marker_fut = pin!(ContainerNodeFuture::new(
            pending::<()>(),
            self.list_end_marker.clone()
        ));
        poll_fn(|cx| {
            let _ = list_end_marker_fut.as_mut().poll(cx);
            Poll::Ready(())
        })
        .await;
        real_containing_node
            .insert_before(&self.list_start_marker, Some(&self.list_end_marker))
            .unwrap_throw();
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
        let guard = MiniScopeGuard(|| {
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
        self.executor.run(list_end_marker_fut).await;
        let _ = guard;
    }
}

fn move_nodes_before(
    container: &web_sys::Node,
    start_marker: &web_sys::Node,
    end_marker: &web_sys::Node,
    after: Option<&web_sys::Node>,
) {
    container.insert_before(start_marker, after).unwrap_throw();
    let mut node = start_marker.next_sibling().unwrap();
    loop {
        container.insert_before(&node, after).unwrap_throw();
        if Object::is(end_marker.as_ref(), node.as_ref()) {
            break;
        }
        node = node.next_sibling().unwrap();
    }
}

struct MiniScopeGuard<F: FnMut()>(F);
impl<F: FnMut()> Drop for MiniScopeGuard<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}
