use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Poll, Waker},
};

use pin_cell::{PinCell, PinMut};
use pin_project_lite::pin_project;

use crate::GiveUnforgettableScope;

pin_project! {
    #[project = InnerProject]
    pub(crate) enum Inner<F: Future> {
        Created {
            #[pin] fut: GiveUnforgettableScope<F>,
            state: CreatedState
        },
        Finished {
            out: Option<F::Output>
        },
        Aborted
    }
}
pub(crate) enum CreatedState {
    NotYetPinned,
    Pinned { local_waker: Waker },
}

impl<F: Future> Future for Inner<F> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match this {
            InnerProject::Created { fut, state } => match state {
                CreatedState::NotYetPinned => {
                    unreachable!("Cannot be spawned before pinned.");
                }
                CreatedState::Pinned { local_waker } => match fut.poll(cx) {
                    Poll::Ready(res) => {
                        let waker = local_waker.to_owned();
                        self.set(Inner::Finished { out: Some(res) });
                        waker.wake();
                        Poll::Ready(())
                    }
                    Poll::Pending => Poll::Pending,
                },
            },
            InnerProject::Finished { .. } => {
                panic!("Polled after Ready returned.");
            }
            InnerProject::Aborted => Poll::Ready(()),
        }
    }
}
pub struct RemoteStaticFuture {
    remote: Pin<Rc<PinCell<dyn Future<Output = ()> + 'static>>>,
}
impl RemoteStaticFuture {
    pub(crate) unsafe fn new<'x, F: Future + 'x>(remote: Pin<Rc<PinCell<Inner<F>>>>) -> Self {
        let remote = remote as Pin<Rc<PinCell<dyn Future<Output = ()> + 'x>>>;
        let remote = unsafe {
            std::mem::transmute::<
                Pin<Rc<PinCell<dyn Future<Output = ()> + 'x>>>,
                Pin<Rc<PinCell<dyn Future<Output = ()> + 'static>>>,
            >(remote)
        };
        Self { remote }
    }
}
impl Future for RemoteStaticFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let cell = self.remote.as_ref();
        let mut bm = cell.borrow_mut();
        let bm = PinMut::as_mut(&mut bm);
        bm.poll(cx)
    }
}
pub(crate) trait InnerTrait {
    type Output;
    fn poll_spawn(
        self: Pin<Rc<Self>>,
        cx: &mut std::task::Context<'_>,
    ) -> PollSpawnResult<Self::Output>;
    fn abort_and_drop(self: Pin<&Self>);
}
pub(crate) enum PollSpawnResult<R> {
    RemoteFuture(RemoteStaticFuture),
    Running,
    Completed(R),
}
impl<F: Future> InnerTrait for PinCell<Inner<F>> {
    type Output = F::Output;

    fn poll_spawn(
        self: Pin<Rc<Self>>,
        cx: &mut std::task::Context<'_>,
    ) -> PollSpawnResult<Self::Output> {
        let mut bm = self.as_ref().borrow_mut();
        let bm = PinMut::as_mut(&mut bm);
        match bm.project() {
            InnerProject::Created { state, .. } => match state {
                CreatedState::NotYetPinned => {
                    *state = CreatedState::Pinned {
                        local_waker: cx.waker().to_owned(),
                    };
                    PollSpawnResult::RemoteFuture(unsafe { RemoteStaticFuture::new(self.clone()) })
                }
                CreatedState::Pinned { .. } => PollSpawnResult::Running,
            },
            InnerProject::Finished { out } => {
                if let Some(res) = out.take() {
                    PollSpawnResult::Completed(res)
                } else {
                    panic!("Polled after Ready returned.");
                }
            }
            InnerProject::Aborted => {
                unreachable!("Cannot be polled after aborted.");
            }
        }
    }

    fn abort_and_drop(self: Pin<&Self>) {
        let mut bm = self.borrow_mut();
        let mut bm = PinMut::as_mut(&mut bm);
        bm.set(Inner::Aborted);
    }
}
