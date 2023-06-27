use super::super::utils::{self, WakerVec};
use crate::{
    context::{DomContext, DOM_CONTEXT},
    dropping::DetachmentBlocker,
    position::ChildPosition,
};

use core::fmt;
use core::future::Future;
use core::mem::MaybeUninit;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::vec::Vec;

use bitvec::vec::BitVec;
use pin_project::{pin_project, pinned_drop};

// For code comments, see the array module.

/// A trait for making CombinatorVec behave as Join/TryJoin/Race/RaceOk.
/// See [super::CombinatorBehaviorArray], which is very similar, for documentation.
pub trait CombinatorBehaviorVec<Fut>
where
    Fut: Future,
{
    type Output;
    type StoredItem;
    fn maybe_return(idx: usize, res: Fut::Output) -> ControlFlow<Self::Output, Self::StoredItem>;
    fn when_completed(vec: Vec<Self::StoredItem>) -> Self::Output;
}

/// See [super::CombinatorArray] for documentation.
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[pin_project(PinnedDrop)]
pub struct CombinatorVec<Fut, B>
where
    Fut: Future,
    B: CombinatorBehaviorVec<Fut>,
{
    pending: usize,
    items: Vec<MaybeUninit<B::StoredItem>>,
    wakers: WakerVec,
    filled: BitVec,
    awake_list_buffer: Vec<usize>,
    #[pin]
    futures: Vec<Fut>,
    detachment_blocker: DetachmentBlocker,
}

impl<Fut, B> CombinatorVec<Fut, B>
where
    Fut: Future,
    B: CombinatorBehaviorVec<Fut>,
{
    pub(crate) fn new(futures: Vec<Fut>) -> Self {
        let len = futures.len();
        CombinatorVec {
            pending: len,
            items: std::iter::repeat_with(MaybeUninit::uninit)
                .take(len)
                .collect(),
            wakers: WakerVec::new(len),
            filled: BitVec::repeat(false, len),
            awake_list_buffer: Vec::new(),
            futures,
            detachment_blocker: DetachmentBlocker,
        }
    }
}

impl<Fut, B> fmt::Debug for CombinatorVec<Fut, B>
where
    Fut: Future + fmt::Debug,
    B: CombinatorBehaviorVec<Fut>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.futures.iter()).finish()
    }
}

impl<Fut, B> Future for CombinatorVec<Fut, B>
where
    Fut: Future,
    B: CombinatorBehaviorVec<Fut>,
{
    type Output = B::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        assert!(
            *this.pending > 0 || this.items.is_empty(),
            "Futures must not be polled after completing"
        );

        {
            let mut readiness = this.wakers.readiness();
            readiness.set_parent_waker(cx.waker());

            this.awake_list_buffer.clone_from(readiness.awake_list());

            readiness.clear();
        }

        for index in this.awake_list_buffer.drain(..) {
            if this.filled[index] {
                continue;
            }
            let fut = utils::get_pin_mut_from_vec(this.futures.as_mut(), index).unwrap();
            let mut cx = Context::from_waker(this.wakers.get(index).unwrap());
            if let Poll::Ready(value) = DOM_CONTEXT.with(|parent: &DomContext| {
                let ctx = DomContext::Child {
                    parent,
                    index: index as _,
                };
                DOM_CONTEXT.set(&ctx, || fut.poll(&mut cx))
            }) {
                match B::maybe_return(index, value) {
                    ControlFlow::Continue(store) => {
                        this.items[index].write(store);
                        this.filled.set(index, true);
                        *this.pending -= 1;
                    }
                    ControlFlow::Break(ret) => {
                        return Poll::Ready(ret);
                    }
                }
            }
        }

        if *this.pending == 0 {
            debug_assert!(
                this.filled.iter().all(|filled| *filled),
                "Future should have reached a `Ready` state"
            );
            this.filled.fill(false);

            // SAFETY: this.pending is only decremented when an item slot is filled.
            // pending reaching 0 means the entire items array is filled.
            //
            // For len > 0, we can only enter this if block once (because the assert at the top),
            // so it is safe to take the data.
            // For len == 0, we can enter this if block many times (in case of poll-after-done),
            // but then the items array is empty anyway so we're fine.
            let items = unsafe {
                let items = core::mem::take(this.items);
                core::mem::transmute::<Vec<MaybeUninit<B::StoredItem>>, Vec<B::StoredItem>>(items)
            };

            Poll::Ready(B::when_completed(items))
        } else {
            Poll::Pending
        }
    }
}

/// Drop the already initialized values on cancellation.
#[pinned_drop]
impl<Fut, B> PinnedDrop for CombinatorVec<Fut, B>
where
    Fut: Future,
    B: CombinatorBehaviorVec<Fut>,
{
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();

        if !this.detachment_blocker.block_until_drop() {
            DOM_CONTEXT.with(|parent: &DomContext| parent.remove_child(ChildPosition::default()));
        }

        for (filled, output) in this.filled.iter().zip(this.items.iter_mut()) {
            if *filled {
                // SAFETY: filled is only set to true for initialized items.
                unsafe { output.assume_init_drop() }
            }
        }
    }
}
