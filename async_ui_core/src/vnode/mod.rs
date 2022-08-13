use std::future::Future;
use std::rc::Rc;

use enum_dispatch::enum_dispatch;
use pin_project_lite::pin_project;
pub mod node_concrete;
pub mod node_context;
pub mod node_pass;
pub mod node_portal;
use crate::context::ContextMap;

use crate::{backend::BackendTrait, position::PositionIndex};

use self::{
    node_concrete::ConcreteNodeVNode, node_context::ContextVNode, node_pass::PassVNode,
    node_portal::PortalVNode,
};

#[enum_dispatch]
pub trait VNodeTrait<B: BackendTrait> {
    fn add_child_node(&self, node: B::Node, position: PositionIndex);
    fn del_child_node(&self, position: PositionIndex);
    fn get_context_map<'s>(&'s self) -> &'s ContextMap;
}

#[enum_dispatch(VNodeTrait<B>)]
pub enum VNode<B: BackendTrait> {
    ConcreteNode(ConcreteNodeVNode<B>),
    Context(ContextVNode<B>),
    Pass(PassVNode<B>),
    Portal(PortalVNode<B>),
}

pin_project! {
    pub struct WithVNode<B: BackendTrait, F: Future> {
        #[pin]
        future: F,
        vnode: Rc<VNode<B>>
    }
}

impl<B: BackendTrait, F: Future> WithVNode<B, F> {
    pub fn new(future: F, vnode: Rc<VNode<B>>) -> Self {
        Self { future, vnode }
    }
}
impl<B: BackendTrait, F: Future> Future for WithVNode<B, F> {
    type Output = F::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        B::get_vnode_key().set(this.vnode, || this.future.poll(cx))
    }
}
