use std::{borrow::Borrow, cell::Ref, task::Waker};

use observables::{Observable, ObservableBase, ObservableBorrow, Version};

use crate::{
    edge::TrackedEdge,
    optional::{OptionalNo, OptionalYes},
    tracked::{Tracked, TrackedNode},
};
pub struct XBowObservable<'a, N>
where
    N: TrackedNode,
{
    tracked: &'a Tracked<N>,
}

impl<'a, N, Z> Observable<Z> for XBowObservable<'a, N>
where
    N: TrackedNode,
    N::Edge: TrackedEdge<Optional = OptionalNo>,
    <N::Edge as TrackedEdge>::Data: Borrow<Z>,
    Z: ?Sized,
{
    fn borrow_observable<'b>(&'b self) -> ObservableBorrow<'b, Z> {
        ObservableBorrow::RefCell(Ref::map(self.tracked.borrow(), Borrow::borrow))
    }
}

impl<'a, N> ObservableBase for XBowObservable<'a, N>
where
    N: TrackedNode,
{
    fn add_waker(&self, waker: Waker) {
        self.tracked.edge.listeners().add_outside_waker(waker);
    }
    fn get_version(&self) -> Version {
        self.tracked.edge.listeners().outside_version()
    }
}
pub struct XBowObservableOrFallback<'a, N>
where
    N: TrackedNode,
{
    tracked: &'a Tracked<N>,
    fallback: <N::Edge as TrackedEdge>::Data,
}

impl<'a, N, Z> Observable<Z> for XBowObservableOrFallback<'a, N>
where
    N: TrackedNode,
    <N::Edge as TrackedEdge>::Data: Borrow<Z>,
    Z: ?Sized,
{
    fn borrow_observable<'b>(&'b self) -> ObservableBorrow<'b, Z> {
        if let Some(b) = self.tracked.borrow_opt() {
            ObservableBorrow::RefCell(Ref::map(b, Borrow::borrow))
        } else {
            ObservableBorrow::Borrow(self.fallback.borrow())
        }
    }
}

impl<'a, N> ObservableBase for XBowObservableOrFallback<'a, N>
where
    N: TrackedNode,
{
    fn add_waker(&self, waker: Waker) {
        self.tracked.edge.listeners().add_outside_waker(waker);
    }
    fn get_version(&self) -> Version {
        self.tracked.edge.listeners().outside_version()
    }
}
impl<N> Tracked<N>
where
    N: TrackedNode,
    N::Edge: TrackedEdge<Optional = OptionalNo>,
{
    pub fn as_observable<'a>(&'a self) -> XBowObservable<'a, N> {
        XBowObservable { tracked: self }
    }
}
impl<N> Tracked<N>
where
    N: TrackedNode,
    N::Edge: TrackedEdge<Optional = OptionalYes>,
    <N::Edge as TrackedEdge>::Data: Default,
{
    pub fn as_observable_or_default<'a>(&'a self) -> XBowObservableOrFallback<'a, N> {
        XBowObservableOrFallback {
            tracked: self,
            fallback: Default::default(),
        }
    }
}

impl<N> Tracked<N>
where
    N: TrackedNode,
    N::Edge: TrackedEdge<Optional = OptionalYes>,
    <N::Edge as TrackedEdge>::Data: Default,
{
    pub fn as_observable_or<'a>(
        &'a self,
        fallback: <N::Edge as TrackedEdge>::Data,
    ) -> XBowObservableOrFallback<'a, N> {
        XBowObservableOrFallback {
            tracked: self,
            fallback,
        }
    }
}
