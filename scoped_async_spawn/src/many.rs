use pin_cell::PinCell;
use pin_project_lite::pin_project;
use pin_weak::rc::PinWeak;
use std::{
    cell::RefCell,
    future::Future,
    marker::{PhantomData, PhantomPinned},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use crate::{
    common::{CreatedState, Inner, InnerTrait, PollSpawnResult},
    pointer::Pointer,
    scope::check_scope,
    GiveUnforgettableScope,
};

type Invariant<'s> = (&'s (), fn(&'s ()));
pub trait ExecutorSpawn: Clone {
    type Task;
    fn spawn_local<F: Future<Output = ()> + 'static>(&self, fut: F) -> Self::Task;
}
pub struct SpawnGuard<'s, S, T>
where
    S: ExecutorSpawn,
    T: 's,
{
    spawned: RefCell<Vec<PinWeak<dyn InnerTrait<Output = T> + 's>>>,
    spawner_function: S,
    _phantom_pin: PhantomPinned,
    _phantom_data: PhantomData<Invariant<'s>>,
}

pin_project! {
    pub struct SpawnedTask<'s, S, T>
    where
        S: ExecutorSpawn
    {
        remote: Pin<Rc<dyn InnerTrait<Output = T> + 's>>,
        spawner_function: S,
        task: Option<S::Task>,
    }
}

impl<'s, S, T> SpawnedTask<'s, S, T>
where
    S: ExecutorSpawn,
{
    pub fn abort(self) {
        self.remote.as_ref().abort_and_drop();
    }
}
impl<'s, S, T> Future for SpawnedTask<'s, S, T>
where
    S: ExecutorSpawn,
{
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.remote.to_owned().poll_spawn(cx) {
            PollSpawnResult::RemoteFuture(rf) => {
                let task = this.spawner_function.spawn_local(rf);
                *this.task = Some(task);
                Poll::Pending
            }
            PollSpawnResult::Running => Poll::Pending,
            PollSpawnResult::Completed(res) => Poll::Ready(res),
        }
    }
}

impl<'s, S, T> SpawnGuard<'s, S, T>
where
    S: ExecutorSpawn,
    T: 's,
{
    pub fn new(spawner: S) -> SpawnGuard<'s, S, T> {
        let spawned = RefCell::new(Vec::new());
        SpawnGuard {
            spawned,
            spawner_function: spawner,
            _phantom_data: PhantomData,
            _phantom_pin: PhantomPinned,
        }
    }
    pub fn spawn_task<F: Future<Output = T> + 's>(
        self: Pin<&mut Self>,
        fut: F,
    ) -> SpawnedTask<'s, S, T> {
        let here = Pointer::new(&*self);
        if !check_scope(here) {
            panic!("Not in scope.");
        }
        let remote = Rc::pin(PinCell::new(Inner::Created {
            fut: unsafe { GiveUnforgettableScope::new(fut) },
            state: CreatedState::NotYetPinned,
        }));
        self.spawned
            .borrow_mut()
            .push(PinWeak::downgrade(remote.clone()));
        SpawnedTask {
            remote,
            spawner_function: self.spawner_function.clone(),
            task: None,
        }
    }
}
