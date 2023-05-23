use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::{pin_project, pinned_drop};

use crate::{
    dropping::UnsetIsDropping, position::ChildPosition, DomContext, NodeGroup, DOM_CONTEXT,
};

/// Future wrapper where anything rendered in its child will appear as child of the node.
/// All common components (`Div`, `Button`, etc.) uses this internally.
#[pin_project(PinnedDrop)]
pub struct ContainerNodeFuture<C> {
    #[pin]
    child_future: C,
    group: NodeGroup,
    container: web_sys::Node,
    add_self: AddSelfMode,
    drop: UnsetIsDropping,
}

enum AddSelfMode {
    ShouldNotAdd,
    ShouldAdd,
    Added,
}

impl<C: Future> ContainerNodeFuture<C> {
    /// Return a future wrapping the given child future.
    /// Any node rendered by the child future will appear inside the given node.
    /// Upon first poll of the future `node` will be added to the parent.
    pub fn new(child_future: C, node: web_sys::Node) -> Self {
        Self {
            child_future,
            group: Default::default(),
            container: node,
            add_self: AddSelfMode::ShouldAdd,
            drop: UnsetIsDropping,
        }
    }
    /// Like `new` but `node` won't be added to the parent (do that manually).
    pub fn new_root(child_future: C, node: web_sys::Node) -> Self {
        Self {
            child_future,
            group: Default::default(),
            container: node,
            add_self: AddSelfMode::ShouldNotAdd,
            drop: UnsetIsDropping,
        }
    }
}
impl<C: Future> Future for ContainerNodeFuture<C> {
    type Output = C::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.add_self {
            AddSelfMode::ShouldAdd => {
                *this.add_self = AddSelfMode::Added;
                DOM_CONTEXT.with(|ctx| {
                    ctx.add_child(ChildPosition::default(), this.container.clone());
                })
            }
            _ => {}
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
        if !self.drop.set_here() {
            DOM_CONTEXT.with(|ctx| {
                ctx.remove_child(ChildPosition::default());
            })
        }
    }
}
