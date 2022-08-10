use std::{marker::PhantomData, ops::Deref, rc::Rc, task::Waker};

use observables::{Observable, ObservableBase, Version};

use crate::{
    borrow_output::{Mutable, NotMutable, XBowBorrow},
    edge::TrackedEdge,
    optional::{OptionalNo, OptionalYes},
    trackable::Trackable,
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
    edge: Rc<<N as TrackedNode>::Edge>,
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
    pub fn borrow<'b>(
        &'b self,
    ) -> XBowBorrow<NotMutable, <N::Edge as TrackedEdge>::BorrowGuard<'b>> {
        XBowBorrow::new_without_check(self.edge.borrow_edge(), NotMutable(PhantomData))
    }
    pub fn borrow_mut<'b>(
        &'b self,
    ) -> XBowBorrow<Mutable<'b, N>, <N::Edge as TrackedEdge>::BorrowMutGuard<'b>> {
        XBowBorrow::new_without_check(self.edge.borrow_edge_mut(), Mutable(&self))
    }
}
impl<N> Tracked<N>
where
    N: TrackedNode,
    N::Edge: TrackedEdge<Optional = OptionalYes>,
{
    pub fn borrow_opt<'b>(
        &'b self,
    ) -> Option<XBowBorrow<NotMutable, <N::Edge as TrackedEdge>::BorrowGuard<'b>>> {
        XBowBorrow::new(self.edge.borrow_edge(), NotMutable(PhantomData))
    }
    pub fn borrow_mut_opt<'b>(
        &'b self,
    ) -> Option<XBowBorrow<Mutable<'b, N>, <N::Edge as TrackedEdge>::BorrowMutGuard<'b>>> {
        XBowBorrow::new(self.edge.borrow_edge_mut(), Mutable(&self))
    }
}

pub type TrackedNodeAlias<T, E> = <T as Trackable<E>>::TrackedNode;
pub type TrackedAlias<T, E> = Tracked<TrackedNodeAlias<T, E>>;

impl<N> ObservableBase<<N::Edge as TrackedEdge>::Data> for Tracked<N>
where
    N: TrackedNode,
{
    fn add_waker(&self, waker: Waker) {
        self.edge.listeners().add_outside_waker(waker);
    }
    fn get_version(&self) -> Version {
        self.edge.listeners().outside_version()
    }
}
impl<N> Observable<<N::Edge as TrackedEdge>::Data> for Tracked<N>
where
    N: TrackedNode,
    N::Edge: TrackedEdge<Optional = OptionalNo>,
{
    fn visit<R, F: FnOnce(&<N::Edge as TrackedEdge>::Data) -> R>(&self, func: F) -> R {
        let b = self.borrow();
        func(&*b)
    }
}
