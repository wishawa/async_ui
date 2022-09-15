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
    inside: RefCell<Inside<B>>,
    context: ContextMap,
}
struct Inside<B: BackendTrait> {
    node: RefNode<B>,
    children: BTreeMap<PositionIndex, B::Node>,
}

impl<B: BackendTrait> ConcreteNodeVNode<B> {
    pub fn new(node: RefNode<B>, context: ContextMap) -> Self {
        Self {
            inside: RefCell::new(Inside {
                node,
                children: BTreeMap::new(),
            }),
            context,
        }
    }
}
pub enum RefNode<B: BackendTrait> {
    Parent { parent: B::Node },
    Sibling { parent: B::Node, sibling: B::Node },
}

impl<B: BackendTrait> VNodeTrait<B> for ConcreteNodeVNode<B> {
    fn add_child_node(&self, mut node: <B as BackendTrait>::Node, position: PositionIndex) {
        let mut inside = self.inside.borrow_mut();
        let Inside {
            node: this_node,
            children: children_map,
        } = &mut *inside;
        let next_node = children_map
            .range(position.clone()..)
            .next()
            .map(|(_k, v)| v);
        match this_node {
            RefNode::Parent { parent } => {
                B::add_child_node(parent, &mut node, next_node);
            }
            RefNode::Sibling { parent, sibling } => {
                B::add_child_node(parent, &mut node, Some(next_node.unwrap_or(sibling)));
            }
        }
        children_map.insert(position, node);
    }

    fn del_child_node(&self, position: PositionIndex) -> B::Node {
        let mut inside = self.inside.borrow_mut();
        let mut removed = inside.children.remove(&position).unwrap();
        match &mut inside.node {
            RefNode::Parent { parent } => B::del_child_node(parent, &mut removed),
            RefNode::Sibling { parent, .. } => B::del_child_node(parent, &mut removed),
        }
        removed
    }

    fn get_context_map<'s>(&'s self) -> &'s ContextMap {
        &self.context
    }
}

enum WithConcreteNodeState<B: BackendTrait> {
    NotStarted { node: RefNode<B> },
    Started { vnode: Rc<VNode<B>> },
    Null,
}
pin_project! {
    pub struct WithConcreteNode<B, F>
    where
        F: Future,
        B: BackendTrait
    {
        #[pin]
        future: F,
        state: WithConcreteNodeState<B>
    }
}

impl<B, F> WithConcreteNode<B, F>
where
    F: Future,
    B: BackendTrait,
{
    pub fn new(future: F, node: RefNode<B>) -> Self {
        Self {
            future,
            state: WithConcreteNodeState::NotStarted { node },
        }
    }
}

impl<B, F> Future for WithConcreteNode<B, F>
where
    F: Future,
    B: BackendTrait,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let vnk = B::get_vnode_key();
        let vnode = match std::mem::replace(this.state, WithConcreteNodeState::Null) {
            WithConcreteNodeState::NotStarted { node } => Rc::new(
                ConcreteNodeVNode::new(node, vnk.with(|vn| vn.get_context_map().to_owned())).into(),
            ),
            WithConcreteNodeState::Started { vnode } => vnode,
            _ => unreachable!(),
        };
        let res = vnk.set(&vnode, || this.future.poll(cx));
        *this.state = WithConcreteNodeState::Started { vnode };
        res
    }
}
