//! Wakers that track when they are woken.
//!
//! By tracking which subfutures have woken, we can avoid having to re-poll N subfutures every time.
//! This tracking is done by a [ReadinessArray]/[ReadinessVec]. These store the indexes of the subfutures that have woken.
//! Each subfuture are given a Waker when polled.
//! This waker must know the index of its corresponding subfuture so that it can update Readiness correctly.
//!

mod array;
mod shared_arc;
mod vec;

pub(crate) use array::*;
pub(crate) use vec::*;
