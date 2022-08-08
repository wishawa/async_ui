use std::{
    cell::RefCell,
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::{backend::BackendTrait, context::ContextMap, position::PositionIndex};

use super::{VNode, VNodeTrait};

pub struct ConcreteNodeVNode<B: BackendTrait> {
    node: RefNode<B>,
    children: RefCell<BTreeMap<PositionIndex, B::Node>>,
    context: ContextMap,
}

impl<B: BackendTrait> ConcreteNodeVNode<B> {
    pub fn new(node: RefNode<B>, context: ContextMap) -> Self {
        Self {
            node,
            children: Default::default(),
            context,
        }
    }
}
pub enum RefNode<B: BackendTrait> {
    Parent { parent: B::Node },
    Sibling { parent: B::Node, sibling: B::Node },
}

impl<B: BackendTrait> VNodeTrait<B> for ConcreteNodeVNode<B> {
    fn add_child_node(&self, node: <B as BackendTrait>::Node, position: PositionIndex) {
        let mut children_map = self.children.borrow_mut();
        let next_node = children_map
            .range(position.clone()..)
            .next()
            .map(|(_k, v)| v);
        match &self.node {
            RefNode::Parent { parent } => {
                B::add_child_node(parent, &node, next_node);
            }
            RefNode::Sibling { parent, sibling } => {
                B::add_child_node(parent, &node, Some(next_node.unwrap_or(sibling)));
            }
        }
        children_map.insert(position, node);
    }

    fn del_child_node(&self, position: PositionIndex) {
        let mut children_map = self.children.borrow_mut();
        let removed = children_map.remove(&position);
        if let Some(removed) = removed {
            match &self.node {
                RefNode::Parent { parent } => B::del_child_node(parent, &removed),
                RefNode::Sibling { parent, .. } => B::del_child_node(parent, &removed),
            }
        }
    }

    fn get_context_map<'s>(&'s self) -> &'s ContextMap {
        &self.context
    }
}

pin_project! {
    pub struct WithNode<B, F>
    where
        F: Future,
        B: BackendTrait
    {
        #[pin]
        future: F,
        vnode: Rc<VNode<B>>
    }
}

impl<B, F> WithNode<B, F>
where
    F: Future,
    B: BackendTrait,
{
    pub fn new(future: F, vnode: Rc<VNode<B>>) -> Self {
        Self { future, vnode }
    }
}

impl<B, F> Future for WithNode<B, F>
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
