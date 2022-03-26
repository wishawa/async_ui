use std::{collections::HashMap, hash::Hash, marker::PhantomPinned, pin::Pin};

use smallvec::SmallVec;

use crate::shared::{check_drop_guarantee, SpawnWrappedFuture};

use super::{spawn, BoxFuture, Task};

pub struct SpawnedFuture<'a> {
    state: SpawnedFutureState<'a>,
}

enum SpawnedFutureState<'a> {
    Created { future: BoxFuture<'a, ()> },
    Spawned { _task: Task },
    Null,
}

impl<'a> SpawnedFutureState<'a> {
    unsafe fn launch(&mut self) {
        let this = self;
        if let Self::Created { future } = std::mem::replace(this, Self::Null) {
            let future =
                unsafe { std::mem::transmute::<BoxFuture<'a, ()>, BoxFuture<'static, ()>>(future) };
            let wrapped = SpawnWrappedFuture::new(future);
            let task = spawn(wrapped);
            *this = Self::Spawned { _task: task };
        }
    }
}

pub struct TaskWrapper<'a>(SpawnedFutureState<'a>);

pin_project_lite::pin_project! {
    pub struct SpawnedTasksContainer<'a> {
        tasks: SmallVec<[TaskWrapper<'a>; 1]>,
        _pin: PhantomPinned
    }
}

impl<'a> SpawnedFuture<'a> {
    pub fn new(future: BoxFuture<'a, ()>) -> Self {
        let state = SpawnedFutureState::Created { future };
        Self { state }
    }
    unsafe fn launch_and_get_task(mut self) -> TaskWrapper<'a> {
        unsafe { self.state.launch() };
        let inner = std::mem::replace(&mut self.state, SpawnedFutureState::Null);
        TaskWrapper(inner)
    }
}
impl SpawnedFuture<'static> {
    pub fn launch(self) -> TaskWrapper<'static> {
        unsafe { self.launch_and_get_task() }
    }
}

impl<'a> SpawnedTasksContainer<'a> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tasks: SmallVec::with_capacity(capacity),
            _pin: PhantomPinned,
        }
    }
    pub fn launch_futures(
        self: &mut Pin<&mut Self>,
        futures: impl Iterator<Item = BoxFuture<'a, ()>>,
    ) {
        check_drop_guarantee::<SpawnedTasksContainer>(&self);
        self.tasks.extend(futures.into_iter().map(|fut| {
            let fut = SpawnedFuture::new(fut);
            unsafe { fut.launch_and_get_task() }
        }));
    }
}

pin_project_lite::pin_project! {
    pub struct DynamicSpawnedTasksContainer<'a, K> {
        tasks: HashMap<K, TaskWrapper<'a>>,
        _pin: PhantomPinned
    }
}
impl<'a, K: Eq + Hash> DynamicSpawnedTasksContainer<'a, K> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tasks: HashMap::with_capacity(capacity),
            _pin: PhantomPinned,
        }
    }
    pub fn launch_futures<I>(self: &mut Pin<&mut Self>, futures: I)
    where
        I: Iterator<Item = (K, BoxFuture<'a, ()>)>,
    {
        check_drop_guarantee(&self);
        self.tasks.extend(futures.into_iter().map(|(k, fut)| {
            let fut = SpawnedFuture::new(fut);
            let task = unsafe { fut.launch_and_get_task() };
            (k, task)
        }));
    }
    pub fn remove_futures<'t, I>(self: &mut Pin<&mut Self>, keys: I)
    where
        I: Iterator<Item = &'t K>,
        K: 't,
    {
        keys.for_each(|key| {
            self.tasks.remove(key);
        });
    }
}
