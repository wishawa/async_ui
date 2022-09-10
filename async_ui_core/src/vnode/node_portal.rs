use std::{
    cell::RefCell,
    collections::BTreeMap,
    future::{Future, IntoFuture},
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::{backend::BackendTrait, context::ContextMap, position::PositionIndex};

use super::{VNode, VNodeTrait};

struct Shared<B: BackendTrait> {
    target: Option<Rc<VNode<B>>>,
    nodes: BTreeMap<PositionIndex, Option<B::Node>>,
}

pub struct PortalVNode<B: BackendTrait> {
    shared: Rc<RefCell<Shared<B>>>,
    context: ContextMap,
}

impl<B: BackendTrait> VNodeTrait<B> for PortalVNode<B> {
    fn add_child_node(&self, node: B::Node, position: PositionIndex) {
        let mut bm = self.shared.borrow_mut();
        let ins = if let Some(target) = bm.target.as_ref() {
            target.add_child_node(node, position.clone());
            None
        } else {
            Some(node)
        };
        bm.nodes.insert(position, ins);
    }

    fn del_child_node(&self, position: PositionIndex) -> B::Node {
        let mut bm = self.shared.borrow_mut();
        let _removed = bm.nodes.remove(&position);
        bm.target.as_ref().unwrap().del_child_node(position)
    }

    fn get_context_map<'s>(&'s self) -> &'s ContextMap {
        &self.context
    }
}

pub struct PortalEntry<B: BackendTrait> {
    shared: Rc<RefCell<Shared<B>>>,
}

impl<B: BackendTrait> PortalEntry<B> {
    pub fn mount<'m, I: IntoFuture>(
        &'m mut self,
        into_future: I,
    ) -> WithPortal<'m, B, I::IntoFuture> {
        WithPortal {
            future: into_future.into_future(),
            state: WithPortalState::Shared(self.shared.clone()),
            _phantom: PhantomData,
        }
    }
}
enum WithPortalState<B: BackendTrait> {
    Shared(Rc<RefCell<Shared<B>>>),
    VNode(Rc<VNode<B>>),
    Null,
}
pin_project! {
    pub struct WithPortal<'m, B: BackendTrait, F: Future> {
        #[pin]
        future: F,
        state: WithPortalState<B>,
        _phantom: PhantomData<&'m ()>
    }
}
impl<'m, B: BackendTrait, F: Future> Future for WithPortal<'m, B, F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let vnode = match std::mem::replace(this.state, WithPortalState::Null) {
            WithPortalState::Shared(shared) => Rc::new(
                (PortalVNode {
                    context: B::get_vnode_key().with(|vn| vn.get_context_map().to_owned()),
                    shared,
                })
                .into(),
            ),
            WithPortalState::VNode(vnode) => vnode,
            WithPortalState::Null => unreachable!(),
        };
        let res = B::get_vnode_key().set(&vnode, || this.future.poll(cx));
        *this.state = WithPortalState::VNode(vnode);
        res
    }
}
pub struct PortalExit<B: BackendTrait> {
    shared: Rc<RefCell<Shared<B>>>,
}

impl<B: BackendTrait> Future for PortalExit<B> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let vnode = B::get_vnode_key().with(Clone::clone);
        let mut bm = self.shared.borrow_mut();
        if bm.target.is_none() {
            bm.nodes.iter_mut().for_each(|(k, v)| {
                vnode.add_child_node(v.take().expect("portal already active"), k.clone());
            });
            bm.target = Some(vnode);
        }
        Poll::Pending
    }
}

impl<B: BackendTrait> Drop for PortalExit<B> {
    fn drop(&mut self) {
        let mut bm = self.shared.borrow_mut();
        if let Some(vn) = bm.target.take() {
            bm.nodes
                .iter_mut()
                .for_each(|(k, v)| *v = Some(vn.del_child_node(k.clone())));
        }
    }
}

pub fn create_portal_pair<B: BackendTrait>() -> (PortalEntry<B>, PortalExit<B>) {
    let shared = Rc::new(RefCell::new(Shared {
        nodes: BTreeMap::new(),
        target: None,
    }));
    (
        PortalEntry {
            shared: shared.clone(),
        },
        PortalExit { shared },
    )
}
