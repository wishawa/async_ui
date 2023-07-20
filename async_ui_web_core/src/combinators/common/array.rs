use crate::{
    context::{DomContext, DOM_CONTEXT},
    dropping::DetachmentBlocker,
    position::ChildPosition,
};

use super::super::utils::{self, WakerArray};

use core::array;
use core::fmt;
use core::future::Future;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};

use pin_project::{pin_project, pinned_drop};

/// A trait for making CombinatorArray behave as Join/TryJoin/Race/RaceOk.
pub trait CombinatorBehaviorArray<Fut, const N: usize>
where
    Fut: Future,
{
    /// Hack to allow racing empty arrays to pend forever.
    const PEND_IF_EMPTY: bool;

    /// The output type of the future.
    ///
    /// Example:
    /// for Join, this is [F::Output; N].
    /// for RaceOk, this is Result<F::Ok, [F::Error; N]>.
    type Output;

    /// The type of item stored.
    ///
    /// Example:
    /// for Join this is F::Output.
    /// for RaceOk this is F::Error.
    type StoredItem;

    /// Takes the output of a subfuture and decide what to do with it.
    /// If this function returns ControlFlow::Break(output), the combinator would early return Poll::Ready(output).
    /// For ControlFlow::Continue(item), the combinator would keep the item in an array.
    /// If by the end, all items are kept (no early return made),
    /// then `when_completed` will be called on the items array.
    ///
    /// Example:
    /// Join will always wrap the output in ControlFlow::Continue because it want to wait until all outputs are ready.
    /// Race will always wrap the output in ControlFlow::Break because it want to early return with the first output.
    fn maybe_return(idx: usize, res: Fut::Output) -> ControlFlow<Self::Output, Self::StoredItem>;

    /// Called when all subfutures are completed and none caused the combinator to return early.
    /// The argument is an array of the kept item from each subfuture.
    fn when_completed(arr: [Self::StoredItem; N]) -> Self::Output;
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
#[pin_project(PinnedDrop)]
pub struct CombinatorArray<Fut, B, const N: usize>
where
    Fut: Future,
    B: CombinatorBehaviorArray<Fut, N>,
{
    behavior: PhantomData<B>,
    /// Number of subfutures that have not yet completed.
    pending: usize,
    wakers: WakerArray<N>,
    /// The stored items from each subfuture.
    items: [MaybeUninit<B::StoredItem>; N],
    /// Whether each item in self.items is initialized.
    /// Invariant: self.filled.count_falses() == self.pending.
    filled: [bool; N],
    /// A temporary buffer for indices that have woken.
    /// The data here don't have to persist between each `poll`.
    awake_list_buffer: [usize; N],
    #[pin]
    futures: [Fut; N],
    detachment_blocker: DetachmentBlocker,
}

impl<Fut, B, const N: usize> CombinatorArray<Fut, B, N>
where
    Fut: Future,
    B: CombinatorBehaviorArray<Fut, N>,
{
    #[inline]
    pub(crate) fn new(futures: [Fut; N]) -> Self {
        CombinatorArray {
            behavior: PhantomData,
            pending: N,
            wakers: WakerArray::new(),
            items: array::from_fn(|_| MaybeUninit::uninit()),
            filled: [false; N],
            // TODO: this is a temporary buffer so it can be MaybeUninit.
            awake_list_buffer: [0; N],
            futures,
            detachment_blocker: DetachmentBlocker,
        }
    }
}

impl<Fut, B, const N: usize> fmt::Debug for CombinatorArray<Fut, B, N>
where
    Fut: Future + fmt::Debug,
    B: CombinatorBehaviorArray<Fut, N>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.futures.iter()).finish()
    }
}

impl<Fut, B, const N: usize> Future for CombinatorArray<Fut, B, N>
where
    Fut: Future,
    B: CombinatorBehaviorArray<Fut, N>,
{
    type Output = B::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        if N == 0 && B::PEND_IF_EMPTY {
            return Poll::Pending;
        }

        // If this.pending == 0, the future is done.
        assert!(
            N == 0 || *this.pending > 0,
            "Futures must not be polled after completing"
        );

        let num_awake = {
            // Lock the readiness Mutex.
            let mut readiness = this.wakers.readiness();
            readiness.set_parent_waker(cx.waker());

            // Copy the list of indices that have woken..
            let awake_list = readiness.awake_list();
            let num_awake = awake_list.len();
            this.awake_list_buffer[..num_awake].copy_from_slice(awake_list);

            // Clear the list.
            readiness.clear();
            num_awake

            // Awakeneess Mutex should be unlocked here.
        };

        // Iterate over the indices we've copied out of the Mutex.
        for &index in this.awake_list_buffer.iter().take(num_awake) {
            let filled = &mut this.filled[index];
            if *filled {
                // Woken subfuture is already complete, don't poll it again.
                // (Futures probably shouldn't wake after they are complete, but there's no guarantee.)
                continue;
            }
            let fut = utils::get_pin_mut(this.futures.as_mut(), index).unwrap();
            let mut cx = Context::from_waker(this.wakers.get(index).unwrap());
            if let Poll::Ready(value) = DOM_CONTEXT.with(|parent: &DomContext| {
                let ctx = DomContext::Child {
                    parent,
                    index: index as _,
                };
                DOM_CONTEXT.set(&ctx, || fut.poll(&mut cx))
            }) {
                match B::maybe_return(index, value) {
                    // Keep the item for returning once every subfuture is done.
                    ControlFlow::Continue(store) => {
                        this.items[index].write(store);
                        *filled = true;
                        *this.pending -= 1;
                    }
                    // Early return.
                    ControlFlow::Break(ret) => return Poll::Ready(ret),
                }
            }
        }

        // Check whether we're all done now or need to keep going.
        if *this.pending == 0 {
            // Check an internal invariant.
            // No matter how ill-behaved the subfutures are, this should be held.
            debug_assert!(
                this.filled.iter().all(|&filled| filled),
                "Future should have filled items array"
            );
            this.filled.fill(false);

            let mut items = array::from_fn(|_| MaybeUninit::uninit());
            core::mem::swap(this.items, &mut items);

            // SAFETY: this.pending is only decremented when an item slot is filled.
            // pending reaching 0 means the entire items array is filled.
            //
            // For N > 0, we can only enter this if block once (because the assert at the top),
            // so it is safe to take the data.
            // For N == 0, we can enter this if block many times (in case of poll-after-done),
            // but then the items array is empty anyway so we're fine.
            let items = unsafe { utils::array_assume_init(items) };

            // Let the Behavior do any final transformation.
            // For example, TryJoin would wrap the whole thing in Ok.
            Poll::Ready(B::when_completed(items))
        } else {
            Poll::Pending
        }
    }
}

/// Drop the already initialized values on cancellation.
#[pinned_drop]
impl<Fut, B, const N: usize> PinnedDrop for CombinatorArray<Fut, B, N>
where
    Fut: Future,
    B: CombinatorBehaviorArray<Fut, N>,
{
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();

        if !this.detachment_blocker.block_until_drop() {
            DOM_CONTEXT.with(|parent: &DomContext| parent.remove_child(ChildPosition::default()));
        }

        for (&filled, output) in this.filled.iter().zip(this.items.iter_mut()) {
            if filled {
                // SAFETY: filled is only set to true for initialized items.
                unsafe { output.assume_init_drop() };
            }
        }
    }
}
