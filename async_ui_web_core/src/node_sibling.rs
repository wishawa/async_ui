use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::{pin_project, pinned_drop};

use crate::{
    dropping::UnsetIsDropping, position::ChildPosition, DomContext, NodeGroup, DOM_CONTEXT,
};

/// Future wrapper where anything rendered in its child will appear as a sibling of a node.
/// This is only used for making insertable lists.
///
/// Everything rendered will appear **before** the reference node.
#[pin_project(PinnedDrop)]
pub struct SiblingNodeFuture<C> {
    #[pin]
    child_future: C,
    group: NodeGroup,
    reference: web_sys::Node,
    drop: UnsetIsDropping,
}

impl<C: Future> SiblingNodeFuture<C> {
    pub fn new(child_future: C, sibling: web_sys::Node) -> Self {
        Self {
            child_future,
            group: Default::default(),
            reference: sibling,
            drop: UnsetIsDropping,
        }
    }
}
impl<C: Future> Future for SiblingNodeFuture<C> {
    type Output = C::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        DOM_CONTEXT.with(|parent: &DomContext| {
            let ctx = DomContext::Sibling {
                group: this.group,
                reference: this.reference,
                parent,
            };
            DOM_CONTEXT.set(&ctx, || this.child_future.poll(cx))
        })
    }
}

#[pinned_drop]
impl<C> PinnedDrop for SiblingNodeFuture<C> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        if !this.drop.set_here() {
            DOM_CONTEXT.with(|parent: &DomContext| {
                (DomContext::Sibling {
                    group: this.group,
                    reference: this.reference,
                    parent,
                })
                .remove_child(ChildPosition::default());
            })
        }
    }
}
