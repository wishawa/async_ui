use super::super::utils::WakerArray;
use crate::{
    context::{DomContext, DOM_CONTEXT},
    dropping::DetachmentBlocker,
    position::ChildPosition,
};

use core::fmt::{self, Debug};
use core::future::Future;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};

/// An internal trait.
/// Join/Race/RaceOk/TryJoin uses this to construct their CombinatorTupleN.
/// This trait is implemented on tuple of the format
/// ((fut1, fut2,...), Behavior ZST, PhantomData<OutputType>).
pub trait CombineTuple {
    /// The resulting combinator future.
    type Combined;
    fn combine(self) -> Self::Combined;
}

/// This and [TupleWhenCompleted] takes the role of [super::array::CombinatorBehaviorArray] but for tuples.
/// Type parameters:
/// R = the return type of a subfuture.
/// O = the return type of the combinator future.
pub trait TupleMaybeReturn<R, O> {
    /// The type of the item to store for this subfuture.
    type StoredItem;
    /// Take the return value of a subfuture and decide whether to store it or early return.
    /// ControlFlow::Continue(v) = store v.
    /// ControlFlow::Break(o) = early return o.
    fn maybe_return(idx: usize, res: R) -> ControlFlow<O, Self::StoredItem>;
}
/// This and [TupleMaybeReturn] takes the role of [super::array::CombinatorBehaviorArray] but for tuples.
/// Type parameters:
/// S = the type of the stored tuples = (F1::StoredItem, F2::StoredItem, ...).
/// O = the return type of the combinator future.
pub trait TupleWhenCompleted<S, O> {
    /// Called when all subfutures are completed and none caused the combinator to return early.
    /// The argument is an array of the kept item from each subfuture.
    fn when_completed(stored_items: S) -> O;
}

macro_rules! impl_common_tuple {
	($mod_name:ident $StructName:ident $($F:ident=$idx:tt)+) => {
        #[allow(non_snake_case)]
		mod $mod_name {
			#[pin_project::pin_project]
            pub(super) struct Futures<$($F,)+> { $(#[pin] pub $F: $F,)+ }
            pub(super) const LEN: usize = [$($idx,)+].len();
		}
		#[pin_project::pin_project(PinnedDrop)]
        #[must_use = "futures do nothing unless you `.await` or poll them"]
        #[allow(non_snake_case)]
        pub struct $StructName<B, O, $($F),+>
        where
            $(
                $F: Future,
            )+
            $(
                B: TupleMaybeReturn<$F::Output, O>,
            )+
            B: TupleWhenCompleted<($(<B as TupleMaybeReturn<$F::Output, O>>::StoredItem,)+), O>
        {
            behavior: PhantomData<B>,
            pending: usize,
            items: ($(MaybeUninit<<B as TupleMaybeReturn<$F::Output, O>>::StoredItem>,)+),
            wakers: WakerArray<{$mod_name::LEN}>,
            filled: [bool; $mod_name::LEN],
            awake_list_buffer: [usize; $mod_name::LEN],
            #[pin]
            futures: $mod_name::Futures<$($F,)+>,
            detachment_blocker: DetachmentBlocker,
        }

        impl<B, O, $($F),+> Debug for $StructName<B, O, $($F),+>
        where
            $(
                $F: Future + Debug,
            )+
            $(
                B: TupleMaybeReturn<$F::Output, O>,
            )+
            B: TupleWhenCompleted<($(<B as TupleMaybeReturn<$F::Output, O>>::StoredItem,)+), O>
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple("Join")
                    $(.field(&self.futures.$F))+
                    .finish()
            }
        }

		impl<B, O, $($F),+> CombineTuple for (($($F,)+), B, PhantomData<O>)
		where
            $(
                $F: Future,
            )+
            $(
                B: TupleMaybeReturn<$F::Output, O>,
            )+
            B: TupleWhenCompleted<($(<B as TupleMaybeReturn<$F::Output, O>>::StoredItem,)+), O>
        {
			type Combined = $StructName<B, O, $($F,)+>;
			fn combine(self) -> Self::Combined {
				$StructName {
                    behavior: PhantomData,
                    filled: [false; $mod_name::LEN],
                    items: ($(MaybeUninit::<<B as TupleMaybeReturn<$F::Output, O>>::StoredItem>::uninit(),)+),
                    wakers: WakerArray::new(),
                    pending: $mod_name::LEN,
                    awake_list_buffer: [0; $mod_name::LEN],
                    futures: $mod_name::Futures {$($F: self.0.$idx,)+},
                    detachment_blocker: DetachmentBlocker
                }
			}
		}

        #[allow(unused_mut)]
        #[allow(unused_parens)]
        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<B, O, $($F),+> Future for $StructName<B, O, $($F),+>
        where
            $(
                $F: Future,
            )+
            $(
                B: TupleMaybeReturn<$F::Output, O>,
            )+
            B: TupleWhenCompleted<($(<B as TupleMaybeReturn<$F::Output, O>>::StoredItem,)+), O>

        {
            type Output = O;

            fn poll(
                self: Pin<&mut Self>, cx: &mut Context<'_>
            ) -> Poll<Self::Output> {
                let mut this = self.project();
                assert!(
                    *this.pending > 0,
                    "Futures must not be polled after completing"
                );

                let mut futures = this.futures.project();

                let num_awake = {
                    let mut readiness = this.wakers.readiness();
                    readiness.set_parent_waker(cx.waker());
                    let awake_list = readiness.awake_list();
                    let num_awake = awake_list.len();
                    this.awake_list_buffer[..num_awake].copy_from_slice(awake_list);
                    readiness.clear();
                    num_awake
                };


                for &index in this.awake_list_buffer.iter().take(num_awake) {
                    let filled = &mut this.filled[index];
                    if *filled {
                        continue;
                    }
                    let mut cx = Context::from_waker(this.wakers.get(index).unwrap());
                    let ready = match index {
                        $(
                            $idx => {
                                if let Poll::Ready(value) = DOM_CONTEXT.with(|parent: &DomContext| {
                                    let ctx = DomContext::Child {
                                        parent,
                                        index: $idx as _,
                                    };
                                    DOM_CONTEXT.set(&ctx, || futures.$F.as_mut().poll(&mut cx))
                                }) {
									match B::maybe_return($idx, value) {
										ControlFlow::Break(ret) => {
                                            return Poll::Ready(ret);
										},
										ControlFlow::Continue(store) => {
											this.items.$idx.write(store);
											true
										}
									}
                                }
                                else {
                                    false
                                }
                            },
                        )*
                        _ => unreachable!()
                    };
                    if ready {
                        *this.pending -= 1;
                        *filled = true;
                    }
                }

                if *this.pending == 0 {
                    debug_assert!(this.filled.iter().all(|&filled| filled), "Future should have filled items array");
                    this.filled.fill(false);
                    let out = {
                        let mut out = ($(MaybeUninit::<<B as TupleMaybeReturn<$F::Output, O>>::StoredItem>::uninit(),)+);
                        core::mem::swap(&mut out, this.items);
                        let ($($F,)+) = out;
                        // SAFETY: we've checked with the state that all of our outputs have been
                        // filled, which means we're ready to take the data and assume it's initialized.
                        unsafe { ($($F.assume_init(),)+) }
                    };
                    Poll::Ready(B::when_completed(out))
                }
                else {
                    Poll::Pending
                }
            }
        }

        #[pin_project::pinned_drop]
        impl<B, O, $($F),+> PinnedDrop for $StructName<B, O, $($F),+>
        where
            $(
                $F: Future,
            )+
            $(
                B: TupleMaybeReturn<$F::Output, O>,
            )+
            B: TupleWhenCompleted<($(<B as TupleMaybeReturn<$F::Output, O>>::StoredItem,)+), O>
        {
            fn drop(self: Pin<&mut Self>) {
                let this = self.project();

                if !this.detachment_blocker.block_until_drop() {
                    DOM_CONTEXT.with(|parent: &DomContext| parent.remove_child(ChildPosition::default()));
                }

                $(
                    if this.filled[$idx] {
                        // SAFETY: we've just filtered down to *only* the initialized values.
                        // We can assume they're initialized, and this is where we drop them.
                        unsafe { this.items.$idx.assume_init_drop() };
                    }
                )+
            }
        }
	};
}

impl_common_tuple! { common1 Combine1 A0=0 }
impl_common_tuple! { common2 Combine2 A0=0 A1=1 }
impl_common_tuple! { common3 Combine3 A0=0 A1=1 A2=2 }
impl_common_tuple! { common4 Combine4 A0=0 A1=1 A2=2 A3=3 }
impl_common_tuple! { common5 Combine5 A0=0 A1=1 A2=2 A3=3 A4=4 }
impl_common_tuple! { common6 Combine6 A0=0 A1=1 A2=2 A3=3 A4=4 A5=5 }
impl_common_tuple! { common7 Combine7 A0=0 A1=1 A2=2 A3=3 A4=4 A5=5 A6=6 }
impl_common_tuple! { common8 Combine8 A0=0 A1=1 A2=2 A3=3 A4=4 A5=5 A6=6 A7=7 }
impl_common_tuple! { common9 Combine9 A0=0 A1=1 A2=2 A3=3 A4=4 A5=5 A6=6 A7=7 A8=8 }
impl_common_tuple! { common10 Combine10 A0=0 A1=1 A2=2 A3=3 A4=4 A5=5 A6=6 A7=7 A8=8 A9=9 }
impl_common_tuple! { common11 Combine11 A0=0 A1=1 A2=2 A3=3 A4=4 A5=5 A6=6 A7=7 A8=8 A9=9 A10=10 }
impl_common_tuple! { common12 Combine12 A0=0 A1=1 A2=2 A3=3 A4=4 A5=5 A6=6 A7=7 A8=8 A9=9 A10=10 A11=11 }
