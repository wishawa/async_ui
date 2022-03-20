use std::{
    collections::HashMap,
    future::Future,
    hash::Hash,
    marker::{PhantomData, PhantomPinned},
    pin::Pin,
    task::{Context, Poll},
};

use smol::Task;

use crate::shared::SpawnWrappedFuture;

use super::super::shared::DROP_GUARANTEED_SCOPED;

use super::{spawn, BoxFuture};

pin_project_lite::pin_project! {
    pub struct SpawnedFuture<'a>
    {
        state: SpawnedFutureState<'a>,
        _phantom: PhantomPinned,
    }
}

enum SpawnedFutureState<'a> {
    Created { future: BoxFuture<'a, ()> },
    Spawned { _task: Task<()> },
    Null,
}

impl<'a> SpawnedFutureState<'a> {
    fn launch(&mut self) {
        let self_loc = &*self as *const _ as *const ();
        let this = self;
        if let Self::Created { future } = std::mem::replace(this, Self::Null) {
            DROP_GUARANTEED_SCOPED.with(|&(target_start, target_end)| {
                if self_loc < target_start || self_loc >= target_end {
                    panic!("spawn without drop guarantee");
                } else {
                    let future = unsafe {
                        std::mem::transmute::<BoxFuture<'a, ()>, BoxFuture<'static, ()>>(future)
                    };
                    let wrapped = SpawnWrappedFuture::new(future);
                    let task = spawn(wrapped);
                    *this = Self::Spawned { _task: task };
                }
            })
        }
    }
}

pub struct TaskWrapper<'a>(SpawnedFutureState<'a>);

impl<'a> SpawnedFuture<'a> {
    pub fn new(future: BoxFuture<'a, ()>) -> Self {
        let state = SpawnedFutureState::Created { future };
        Self {
            state,
            _phantom: PhantomPinned,
        }
    }
    pub unsafe fn launch_and_get_task(&mut self) -> TaskWrapper<'a> {
        self.state.launch();
        let inner = std::mem::replace(&mut self.state, SpawnedFutureState::Null);
        TaskWrapper(inner)
    }
}

// impl<'a> Future for SpawnedFuture<'a> {
//     type Output = ();
//     fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let this = self.project();
//         this.state.launch();
//         Poll::Pending
//     }
// }
