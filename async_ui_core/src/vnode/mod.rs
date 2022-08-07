use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use enum_dispatch::enum_dispatch;
pub mod concrete_node;
pub mod pass;
use pin_project_lite::pin_project;

use crate::{backend::BackendTrait, position::PositionIndex};

use self::{concrete_node::ConcreteNodeVNode, pass::PassVNode};

#[enum_dispatch]
trait VNodeTrait<B: BackendTrait> {
    fn add_child_node(&self, node: B::Node, position: PositionIndex);
    fn del_child_node(&self, position: PositionIndex);
}

#[enum_dispatch(VNodeTrait<B>)]
pub enum VNode<B: BackendTrait> {
    ConcreteNode(ConcreteNodeVNode<B>),
    Pass(PassVNode<B>),
}

pin_project! {
    pub struct GiveVNode<F, B>
    where
        F: Future,
        B: BackendTrait
    {
        #[pin]
        future: F,
        vnode: Rc<VNode<B>>
    }
}

impl<F, B> GiveVNode<F, B>
where
    F: Future,
    B: BackendTrait,
{
    pub fn new(future: F, vnode: Rc<VNode<B>>) -> Self {
        Self { future, vnode }
    }
}

impl<F, B> Future for GiveVNode<F, B>
where
    F: Future,
    B: BackendTrait,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        B::get_vnode_key().set(this.vnode, || this.future.poll(cx))
    }
}
