use std::{cell::Ref, ops::Deref, rc::Rc};

use crate::{
    edge::TrackedEdge, notify_guard::NotifyGuard, optional::OptionalNo, trackable::Trackable,
};

pub trait TrackedNode {
    type Edge: TrackedEdge;
    fn new(edge: Rc<Self::Edge>) -> Self;
    fn invalidate_outside_down(&self);
}
pub struct Tracked<N>
where
    N: TrackedNode,
{
    inner: N,
    pub(crate) edge: Rc<<N as TrackedNode>::Edge>,
}

impl<N> Deref for Tracked<N>
where
    N: TrackedNode,
{
    type Target = N;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<N> Tracked<N>
where
    N: TrackedNode,
{
    pub fn create_with_edge(edge: Rc<N::Edge>) -> Self {
        let inner = TrackedNode::new(edge.clone());
        Self { inner, edge }
    }
    pub fn invalidate_inside_up(&self) {
        self.edge.invalidate_inside_up();
    }
    pub fn invalidate_outside_down(&self) {
        self.edge.invalidate_outside_here();
        self.inner.invalidate_outside_down();
    }
}
impl<N> Tracked<N>
where
    N: TrackedNode,
    N::Edge: TrackedEdge<Optional = OptionalNo>,
{
    pub fn borrow<'b>(&'b self) -> Ref<'b, <N::Edge as TrackedEdge>::Data> {
        self.borrow_opt().unwrap()
    }
    pub fn borrow_mut<'b>(&'b self) -> NotifyGuard<'b, N> {
        self.borrow_mut_opt().unwrap()
    }
}
impl<N> Tracked<N>
where
    N: TrackedNode,
    N::Edge: TrackedEdge,
{
    pub fn borrow_opt<'b>(&'b self) -> Option<Ref<'b, <N::Edge as TrackedEdge>::Data>> {
        self.edge.borrow_edge()
    }
    pub fn borrow_mut_opt<'b>(&'b self) -> Option<NotifyGuard<'b, N>> {
        let inside = self.edge.borrow_edge_mut();
        inside.map(|inside| NotifyGuard {
            inside,
            tracked: self,
        })
    }
}

pub type TrackedNodeAlias<T, E> = <T as Trackable<E>>::TrackedNode;
pub type TrackedAlias<T, E> = Tracked<TrackedNodeAlias<T, E>>;
