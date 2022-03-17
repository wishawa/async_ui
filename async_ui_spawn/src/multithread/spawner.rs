use std::{
    future::Future,
    marker::PhantomPinned,
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
    Spawned,
}

impl<'a> SpawnedFuture<'a> {
    pub fn new(future: BoxFuture<'a, ()>) -> Self {
        let state = SpawnedFutureState::Created { future };
        Self {
            state,
            _phantom: PhantomPinned,
        }
    }
}

impl<'a> Future for SpawnedFuture<'a> {
    type Output = Task<()>;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let self_loc = &*self as *const _ as *const ();
        let this = self.project();
        if let SpawnedFutureState::Created { future } =
            std::mem::replace(this.state, SpawnedFutureState::Spawned)
        {
            DROP_GUARANTEED_SCOPED.with(|&(target_start, target_end)| {
                if self_loc < target_start || self_loc >= target_end {
                    panic!("spawn without drop guarantee");
                } else {
                    Poll::Ready({
                        let future = unsafe {
                            std::mem::transmute::<BoxFuture<'a, ()>, BoxFuture<'static, ()>>(future)
                        };
                        let wrapped = SpawnWrappedFuture::new(future);
                        spawn(wrapped)
                    })
                }
            })
        } else {
            panic!("spawn polled after completed")
        }
    }
}
