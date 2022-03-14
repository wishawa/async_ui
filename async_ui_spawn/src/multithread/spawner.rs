use std::{
    future::Future,
    marker::PhantomPinned,
    pin::Pin,
    task::{Context, Poll},
};

use smol::Task;

use crate::shared::{SpawnContext, SpawnWrappedFuture};

use super::super::shared::DROP_GUARANTEED_SCOPED;

use super::{spawn, BoxFuture, Send};

pin_project_lite::pin_project! {
    pub struct SpawnedFuture<'a, C>
    where C: SpawnContext, C: Send
    {
        state: SpawnedFutureState<'a, C>,
        _phantom: PhantomPinned,
    }
}

enum SpawnedFutureState<'a, C: SpawnContext + Send> {
    Created {
        future: BoxFuture<'a, ()>,
        context: C,
    },
    Spawned,
}

impl<'a, C: SpawnContext + Send> SpawnedFuture<'a, C> {
    pub fn new(future: BoxFuture<'a, ()>, context: C) -> Self {
        let state = SpawnedFutureState::Created { future, context };
        Self {
            state,
            _phantom: PhantomPinned,
        }
    }
}

impl<'a, C: SpawnContext + Send> Future for SpawnedFuture<'a, C> {
    type Output = Task<()>;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let self_loc = &*self as *const _ as *const ();
        let this = self.project();
        if let SpawnedFutureState::Created { future, context } =
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
                        let wrapped = SpawnWrappedFuture::new(future, context);
                        spawn(wrapped)
                    })
                }
            })
        } else {
            panic!("spawn polled after completed")
        }
    }
}
