mod button;
mod text;
pub use button::Button;
pub use text::Text;
mod dummy;
mod event_channel;
use async_ui_core::{
    backend::BackendTrait,
    vnode::{
        node_concrete::{ConcreteNodeVNode, RefNode},
        VNode, VNodeTrait,
    },
};
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use crate::{backend::Backend, widget::WrappedWidget};

pin_project! {
    pub struct ElementFuture<F: Future> {
        #[pin]
        future: F,
        inner: ElementFutureInner
    }
}
struct ElementFutureInner {
    node: WrappedWidget,
    vnodes: Option<MyAndParentVNodes>,
}
struct MyAndParentVNodes {
    my: Rc<VNode<Backend>>,
    parent: Rc<VNode<Backend>>,
}

impl Drop for ElementFutureInner {
    fn drop(&mut self) {
        if let Some(MyAndParentVNodes { parent, .. }) = &self.vnodes {
            parent.del_child_node(Default::default());
        }
    }
}
impl<F: Future> ElementFuture<F> {
    fn new(future: F, node: WrappedWidget) -> Self {
        Self {
            future,
            inner: ElementFutureInner { node, vnodes: None },
        }
    }
}
impl<F: Future> Future for ElementFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let vnk = Backend::get_vnode_key();
        let vnodes = this.inner.vnodes.get_or_insert_with(|| {
            let parent_vnode = vnk.with(Clone::clone);
            parent_vnode.add_child_node(this.inner.node.to_owned(), Default::default());
            let parent_context = parent_vnode.get_context_map().clone();
            let my = Rc::new(
                ConcreteNodeVNode::new(
                    RefNode::Parent {
                        parent: this.inner.node.clone(),
                    },
                    parent_context,
                )
                .into(),
            );
            MyAndParentVNodes {
                my,
                parent: parent_vnode,
            }
        });
        vnk.set(&vnodes.my, || this.future.poll(cx))
    }
}
