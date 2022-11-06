use pin_cell::PinCell;
use pin_project_lite::pin_project;
use pin_weak::rc::PinWeak;
use smallvec::SmallVec;
use std::{
    future::Future,
    marker::{PhantomData, PhantomPinned},
    panic::AssertUnwindSafe,
    pin::Pin,
    rc::Rc,
};

use crate::{
    common::{WrappedFuture, WrappedFutureTrait},
    pointer::Pointer,
    scope::check_scope,
    GiveUnforgettableScope, RemoteStaticFuture,
};

type Invariant<'s> = (&'s (), fn(&'s ()));

struct SpawnedTracker<'s>(SmallVec<[PinWeak<dyn WrappedFutureTrait + 's>; 1]>);

pin_project! {
    /**
     * The guard that ends spawned tasks before they go beyond their lifetime.
     *
     * This must be pinned before use.
     */
    pub struct SpawnGuard<'s> {
        spawned: SpawnedTracker<'s>,
        #[pin]
        _phantom_pin: PhantomPinned,
        _phantom_data: PhantomData<Invariant<'s>>,
    }
}

impl<'s> SpawnGuard<'s> {
    /**
     * Construct a new guard.
     */
    pub fn new() -> Self {
        let spawned = SpawnedTracker(SmallVec::new());
        Self {
            spawned,
            _phantom_data: PhantomData,
            _phantom_pin: PhantomPinned,
        }
    }
    /**
     * Convert a non-'static future to a 'static one.
     *
     * The guard keeps access to the future,
     * and will end the future before it could go beyond its lifetime.
     *
     * The future should not panic on drop. Doing so will cause process abort.
     *
     * If you want to create a guard within the future, make sure that guard last across a yield point.
     * Otherwise that guard will panic [see issue](https://github.com/wishawa/async_ui/issues/6).
     */
    pub fn convert_future<F: Future + 's>(
        self: Pin<&mut Self>,
        fut: F,
    ) -> RemoteStaticFuture<F::Output> {
        let here = Pointer::new(&*self);
        if !check_scope(here) {
            panic!("Not in scope.");
        }
        let remote = Rc::pin(PinCell::new(WrappedFuture::Running {
            fut: unsafe { GiveUnforgettableScope::new(fut) },
        }));
        let this = self.project();
        this.spawned.0.push(PinWeak::downgrade(remote.clone()));
        unsafe { RemoteStaticFuture::new(remote) }
    }
    /**
     * Clear completed future from the guard.
     *
     * The guard keeps track of all spawned future, so it can grow as you spawn more and more.
     * This method removes the guard's reference to completed futures.
     */
    pub fn clear_dead_futures(self: Pin<&mut Self>) {
        let this = self.project();
        this.spawned.0.retain(|el| PinWeak::strong_count(el) > 0);
    }
}

impl<'s> Drop for SpawnedTracker<'s> {
    fn drop(&mut self) {
        let res = std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.0.drain(..).for_each(|ch| {
                if let Some(ch) = ch.upgrade() {
                    ch.as_ref().drop_future_now();
                }
            })
        }));
        if res.is_err() {
            std::process::abort();
        }
    }
}
