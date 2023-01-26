pub mod executor;
pub mod fragment;
mod position;
pub mod window;

use std::{
    cell::RefCell,
    collections::BTreeMap,
    future::Future,
    ops::RangeBounds,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::{pin_project, pinned_drop};
use position::ChildPosition;

enum DomContext<'p> {
    Container {
        group: &'p NodeGroup,
        container: &'p web_sys::Node,
    },
    Sibling {
        group: &'p NodeGroup,
        reference: &'p web_sys::Node,
        container: &'p web_sys::Node,
    },
    Child {
        parent: &'p Self,
        index: usize,
    },
}

scoped_tls_hkt::scoped_thread_local!(
    static DOM_CONTEXT: for<'p> &'p DomContext<'p>
);

type NodeGroup = RefCell<BTreeMap<ChildPosition, web_sys::Node>>;

/// Get the parent where our stuff would be rendered in.
pub fn get_containing_node() -> web_sys::Node {
    DOM_CONTEXT.with(|ctx| ctx.get_containing_node())
}

// BTreeMap doesn't have a drain range method (see Rust issue 81074).
fn drain_btree_for_each<K: Ord + Clone, V, R: RangeBounds<K> + Clone, F: FnMut(V)>(
    btree: &mut BTreeMap<K, V>,
    range: R,
    mut foreach: F,
) {
    while let Some((k, _v)) = btree.range(range.clone()).next_back() {
        let k = k.to_owned();
        let removed = btree.remove(&k).unwrap();
        foreach(removed);
    }
}

impl<'p> DomContext<'p> {
    fn get_containing_node(&self) -> web_sys::Node {
        match self {
            DomContext::Container { container, .. } | DomContext::Sibling { container, .. } => {
                (*container).to_owned()
            }
            DomContext::Child { parent, .. } => parent.get_containing_node(),
        }
    }
    fn add_child(&self, mut position: ChildPosition, new_child: web_sys::Node) {
        match self {
            DomContext::Container { group, container } => {
                let mut group = group.borrow_mut();
                let reference_sibling = group.range((&position)..).next().map(|(_k, v)| v);
                container
                    .insert_before(&new_child, reference_sibling)
                    .expect("insert failed");
                group.insert(position, new_child);
            }
            DomContext::Sibling {
                group,
                container,
                reference,
            } => {
                let mut group = group.borrow_mut();
                let reference_sibling = group
                    .range((&position)..)
                    .next()
                    .map(|(_k, v)| v)
                    .unwrap_or(*reference);
                container
                    .insert_before(&new_child, Some(reference_sibling))
                    .expect("insert failed");
                group.insert(position, new_child);
            }
            DomContext::Child { parent, index } => {
                position.wrap(*index);
                parent.add_child(position, new_child);
            }
        }
    }
    fn remove_child(&self, mut position: ChildPosition) {
        match self {
            DomContext::Container { group, container } => {
                let mut group = group.borrow_mut();
                let next_sib = position.next_sibling();
                drain_btree_for_each(&mut group, (&position)..(&next_sib), |child| {
                    container.remove_child(&child).expect("child disappeared");
                })
            }
            DomContext::Sibling {
                group, container, ..
            } => {
                let mut group = group.borrow_mut();
                let next_sib = position.next_sibling();
                drain_btree_for_each(&mut group, (&position)..(&next_sib), |child| {
                    container.remove_child(&child).expect("child disappeared");
                })
            }
            DomContext::Child { parent, index } => {
                position.wrap(*index);
                parent.remove_child(position);
            }
        }
    }
}

/// Future wrapper where anything rendered in its child future
/// will appear as child of the node.
#[pin_project(PinnedDrop)]
pub struct ContainerNodeFuture<C> {
    #[pin]
    child_future: C,
    group: NodeGroup,
    container: web_sys::Node,
    first: bool,
}

impl<C: Future> ContainerNodeFuture<C> {
    /// Return a future wrapping the given child future.
    /// Any node rendered by the child future will appear inside the given node.
    pub fn new(child_future: C, node: web_sys::Node) -> Self {
        Self {
            child_future,
            group: Default::default(),
            container: node,
            first: false,
        }
    }
}
impl<C: Future> Future for ContainerNodeFuture<C> {
    type Output = C::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if *this.first {
            *this.first = false;
            DOM_CONTEXT.with(|ctx| {
                ctx.add_child(ChildPosition::default(), this.container.clone());
            })
        }
        let ctx = DomContext::Container {
            group: this.group,
            container: this.container,
        };
        DOM_CONTEXT.set(&ctx, || this.child_future.poll(cx))
    }
}

#[pinned_drop]
impl<C> PinnedDrop for ContainerNodeFuture<C> {
    fn drop(self: Pin<&mut Self>) {
        if self.container.is_connected() {
            DOM_CONTEXT.with(|ctx| {
                ctx.remove_child(ChildPosition::default());
            })
        }
    }
}

#[pin_project(PinnedDrop)]
pub struct SiblingNodeFuture<C> {
    #[pin]
    child_future: C,
    group: NodeGroup,
    container: web_sys::Node,
    reference: web_sys::Node,
}

impl<C: Future> SiblingNodeFuture<C> {
    pub fn new(child_future: C, sibling: web_sys::Node, container: web_sys::Node) -> Self {
        Self {
            child_future,
            group: Default::default(),
            container,
            reference: sibling,
        }
    }
}
impl<C: Future> Future for SiblingNodeFuture<C> {
    type Output = C::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let ctx = DomContext::Sibling {
            group: this.group,
            reference: this.reference,
            container: this.container,
        };
        DOM_CONTEXT.set(&ctx, || this.child_future.poll(cx))
    }
}

#[pinned_drop]
impl<C> PinnedDrop for SiblingNodeFuture<C> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        if this.reference.is_connected() {
            for (_k, v) in this.group.borrow_mut().iter() {
                this.container.remove_child(v).expect("child disappeared");
            }
        }
    }
}

#[pin_project]
pub struct ChildFuture<C> {
    #[pin]
    child_future: C,
    index: usize,
}

impl<C: Future> ChildFuture<C> {
    pub fn new(child_future: C, index: usize) -> Self {
        Self {
            child_future,
            index,
        }
    }
}

impl<C: Future> Future for ChildFuture<C> {
    type Output = C::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        DOM_CONTEXT.with(|parent: &DomContext| {
            let ctx = DomContext::Child {
                parent,
                index: *this.index,
            };
            DOM_CONTEXT.set(&ctx, || this.child_future.poll(cx))
        })
    }
}
