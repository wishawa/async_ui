use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use async_ui_core::{
    backend::BackendTrait,
    position::PositionIndex,
    vnode::{
        node_concrete::{ConcreteNodeVNode, RefNode},
        VNode, VNodeTrait,
    },
};
use pin_project_lite::pin_project;
use web_sys::Node;
mod button;
mod event_handler;
mod text;
mod view;
pub use text::Text;
pub use button::Button;
pub use view::View;

use crate::{backend::Backend, window::DOCUMENT};

pin_project! {
    pub struct ElementFuture<F: Future> {
        node: Node,
        #[pin]
        future: F,
        vnode: Option<Rc<VNode<Backend>>>
    }
}
impl<F: Future> ElementFuture<F> {
    fn new(future: F, node: Node) -> Self {
        Self {
            node,
            future,
            vnode: None,
        }
    }
}
impl<F: Future> Future for ElementFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let vnk = Backend::get_vnode_key();
        let my_vnode = this.vnode.get_or_insert_with(|| {
            let parent_context = vnk.with(|parent_vnode| {
                parent_vnode.add_child_node(this.node.to_owned(), PositionIndex::default());
                parent_vnode.get_context_map().clone()
            });
            Rc::new(
                ConcreteNodeVNode::new(
                    RefNode::Parent {
                        parent: this.node.clone(),
                    },
                    parent_context,
                )
                .into(),
            )
        });
        vnk.set(my_vnode, || this.future.poll(cx))
    }
}
fn create_element_future<F: Future>(fut: F, name: &'static str) -> ElementFuture<F> {
    ElementFuture::new(
        fut,
        DOCUMENT
            .with(|doc| doc.create_element(name).expect("create element failed"))
            .into(),
    )
}
