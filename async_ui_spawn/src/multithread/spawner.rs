use crate::shared::SpawnWrappedFuture;

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

impl<'a> SpawnedFuture<'a> {
    pub fn new(future: BoxFuture<'a, ()>) -> Self {
        let state = SpawnedFutureState::Created { future };
        Self { state }
    }
    pub unsafe fn launch_and_get_task(&mut self) -> TaskWrapper<'a> {
        unsafe { self.state.launch() };
        let inner = std::mem::replace(&mut self.state, SpawnedFutureState::Null);
        TaskWrapper(inner)
    }
}
