use std::{future::Future, marker::PhantomPinned, pin::Pin, rc::Rc, task::Poll};

use crate::common::{InnerTrait, PollSpawnResult};
use crate::scope::check_scope;
use pin_cell::PinCell;
use pin_project_lite::pin_project;

use super::common::{CreatedState, Inner, RemoteStaticFuture};
use super::pointer::Pointer;
use super::scope::GiveUnforgettableScope;

pin_project! {
    pub struct SpawnedFuture<F: Future, S: Fn(RemoteStaticFuture) -> O, O> {
        remote: DropWrappedRemote<PinCell<Inner<F>>>,
        spawn_fn: S,
        task: Option<O>,
        _phantom: PhantomPinned
    }
}
struct DropWrappedRemote<I: InnerTrait> {
    remote: Pin<Rc<I>>,
}
impl<I: InnerTrait> Drop for DropWrappedRemote<I> {
    fn drop(&mut self) {
        self.remote.as_ref().abort_and_drop();
    }
}

impl<F: Future, S: Fn(RemoteStaticFuture) -> O, O> SpawnedFuture<F, S, O> {
    pub fn new(fut: F, spawn_fn: S) -> Self {
        let remote = Rc::pin(PinCell::new(Inner::Created {
            fut: unsafe { GiveUnforgettableScope::new(fut) },
            state: CreatedState::NotYetPinned,
        }));
        let remote = DropWrappedRemote { remote };
        let task = None;
        Self {
            remote,
            spawn_fn,
            task,
            _phantom: PhantomPinned,
        }
    }
}
impl<F: Future, S: Fn(RemoteStaticFuture) -> O, O> Future for SpawnedFuture<F, S, O> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let check_ptr = Pointer::new(&*self);
        let this = self.project();

        let in_scope = check_scope(check_ptr);
        if !in_scope {
            panic!("Not in scope.");
        }
        match this.remote.remote.to_owned().poll_spawn(cx) {
            PollSpawnResult::RemoteFuture(rf) => {
                let task = (this.spawn_fn)(rf);
                *this.task = Some(task);
                Poll::Pending
            }
            PollSpawnResult::Running => Poll::Pending,
            PollSpawnResult::Completed(re) => Poll::Ready(re),
        }
    }
}
