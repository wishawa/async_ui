use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    task::Waker,
};

use observables::{Observable, ObservableBase, ObservableBorrow, Version};

use crate::{
    __private_macro_only::OptionalYes,
    edge::TrackedEdge,
    optional::OptionalNo,
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
    fn get_borrow<'b>(&'b self) -> ObservableBorrow<'b, Z> {
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
    fallback: RefCell<<N::Edge as TrackedEdge>::Data>,
}

impl<'a, N, Z> Observable<Z> for XBowObservableOrFallback<'a, N>
where
    N: TrackedNode,
    <N::Edge as TrackedEdge>::Data: Borrow<Z>,
    Z: ?Sized,
{
    fn get_borrow<'b>(&'b self) -> ObservableBorrow<'b, Z> {
        if let Some(b) = self.tracked.borrow_opt() {
            ObservableBorrow::RefCell(Ref::map(b, Borrow::borrow))
        } else {
            ObservableBorrow::RefCell(Ref::map(self.fallback.borrow(), Borrow::borrow))
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
    pub fn to_observable<'a>(&'a self) -> XBowObservable<'a, N> {
        XBowObservable { tracked: self }
    }
}
impl<N> Tracked<N>
where
    N: TrackedNode,
    N::Edge: TrackedEdge<Optional = OptionalYes>,
    <N::Edge as TrackedEdge>::Data: Default,
{
    pub fn to_observable_or_default<'a>(&'a self) -> XBowObservableOrFallback<'a, N> {
        XBowObservableOrFallback {
            tracked: self,
            fallback: RefCell::new(Default::default()),
        }
    }
}
