use pin_cell::PinCell;
use pin_project_lite::pin_project;
use pin_weak::rc::PinWeak;
use smallvec::SmallVec;
use std::{
    future::Future,
    marker::{PhantomData, PhantomPinned},
    pin::Pin,
    rc::Rc,
};

use crate::{
    common::{Inner, InnerTrait},
    pointer::Pointer,
    scope::check_scope,
    GiveUnforgettableScope, RemoteStaticFuture,
};

type Invariant<'s> = (&'s (), fn(&'s ()));

struct SpawnedTracker<'s>(SmallVec<[PinWeak<dyn InnerTrait + 's>; 1]>);
pin_project! {
    pub struct SpawnGuard<'s> {
        spawned: SpawnedTracker<'s>,
        #[pin]
        _phantom_pin: PhantomPinned,
        _phantom_data: PhantomData<Invariant<'s>>,
    }
}

impl<'s> SpawnGuard<'s> {
    pub fn new() -> Self {
        let spawned = SpawnedTracker(SmallVec::new());
        Self {
            spawned,
            _phantom_data: PhantomData,
            _phantom_pin: PhantomPinned,
        }
    }
    pub fn convert_future<F: Future + 's>(
        self: Pin<&mut Self>,
        fut: F,
    ) -> RemoteStaticFuture<F::Output> {
        let here = Pointer::new(&*self);
        if !check_scope(here) {
            panic!("Not in scope.");
        }
        let remote = Rc::pin(PinCell::new(Inner::Running {
            fut: unsafe { GiveUnforgettableScope::new(fut) },
        }));
        let this = self.project();
        this.spawned.0.push(PinWeak::downgrade(remote.clone()));
        unsafe { RemoteStaticFuture::new(remote) }
    }
}

impl<'s> Drop for SpawnedTracker<'s> {
    fn drop(&mut self) {
        self.0.drain(..).for_each(|ch| {
            if let Some(ch) = ch.upgrade() {
                ch.as_ref().abort();
            }
        })
    }
}
