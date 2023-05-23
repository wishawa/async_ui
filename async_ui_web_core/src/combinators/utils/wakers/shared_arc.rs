//! To save on allocations, we avoid making a separate Arc Waker for every subfuture.
//! Rather, we have all N Wakers share a single Arc, and use a "redirect" mechanism to allow different wakers to be distinguished.
//! The mechanism works as follows.
//! The Arc contains 2 things:
//! - the Readiness structure ([ReadinessArray][super::array::ReadinessArray] / [ReadinessVec][super::vec::ReadinessVec])
//! - the redirect array.
//! The redirect array contains N repeated copies of the pointer to the Arc itself (obtained by `Arc::into_raw`).
//! The Waker for the `i`th subfuture points to the `i`th item in the redirect array.
//! (i.e. the Waker pointer is `*const *const A` where `A` is the type of the item in the Arc)
//! When the Waker is woken, we deref it twice (giving reference to the content of the Arc),
//! and compare it to the address of the redirect slice.
//! The difference tells us the index of the waker. We can then record this woken index in the Readiness.
//!
//! ```text
//!    ┌───────────────────────────┬──────────────┬──────────────┐
//!    │                           │              │              │
//!    │    / ┌─────────────┬──────┼───────┬──────┼───────┬──────┼───────┬─────┐ \
//!    ▼   /  │             │      │       │      │       │      │       │     │  \
//!   Arc <   │  Readiness  │  redirect[0] │  redirect[1] │  redirect[2] │ ... │   >
//!    ▲   \  │             │              │              │              │     │  /
//!    │    \ └─────────────┴──────▲───────┴──────▲───────┴──────▲───────┴─────┘ /
//!    │                           │              │              │
//!    └─┐         ┌───────────────┘              │              │
//!      │         │                              │              │
//!      │         │           ┌──────────────────┘              │
//!      │         │           │                                 │
//!      │         │           │           ┌─────────────────────┘
//!      │         │           │           │
//!      │         │           │           │
//! ┌────┼────┬────┼──────┬────┼──────┬────┼──────┬─────┐
//! │    │    │    │      │    │      │    │      │     │
//! │         │ wakers[0] │ wakers[1] │ wakers[2] │ ... │
//! │         │           │           │           │     │
//! └─────────┴───────────┴───────────┴───────────┴─────┘
//! ```

// TODO: Right now each waker gets its own redirect slot.
// We can save space by making size_of::<*const _>() wakers share the same slot.
// With such change, in 64-bit system, the redirect array/vec would only need ⌈N/8⌉ slots instead of N.

use core::task::{RawWaker, RawWakerVTable, Waker};
use std::sync::Arc;

/// A trait to be implemented on [super::WakerArray] and [super::WakerVec] for polymorphism.
/// These are the type that goes in the Arc. They both contain the Readiness and the redirect array/vec.
/// # Safety
/// The `get_redirect_slice` method MUST always return the same slice for the same self.
pub(super) unsafe trait SharedArcContent {
    /// Get the reference of the redirect slice.
    fn get_redirect_slice(&self) -> &[*const Self];

    /// Called when the subfuture at the specified index should be polled.
    /// Should call `Readiness::set_ready`.
    fn wake_index(&self, index: usize);
}

/// Create one waker following the mechanism described in the [module][self] doc.
/// For safety, the index MUST be within bounds of the slice returned by `A::get_redirect_slice()`.
#[deny(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn waker_from_redirect_position<A: SharedArcContent>(
    arc: Arc<A>,
    index: usize,
) -> Waker {
    // For `create_waker`, `wake_by_ref`, `wake`, and `drop_waker`, the following MUST be upheld for safety:
    // - `pointer` must points to a slot in the redirect array.
    // - that slot must contain a pointer of an Arc obtained from `Arc::<A>::into_raw`.
    // - that Arc must still be alive (strong count > 0) at the time the function is called.

    /// Clone a Waker from a type-erased pointer.
    /// The pointer must satisfy the safety constraints listed in the code comments above.
    unsafe fn clone_waker<A: SharedArcContent>(pointer: *const ()) -> RawWaker {
        // Retype the type-erased pointer.
        let pointer = pointer as *const *const A;

        // Increment the count so that the Arc won't die before this new Waker we're creating.
        // SAFETY: The required constraints means
        // - `*pointer` is an `*const A` obtained from `Arc::<A>::into_raw`.
        // - the Arc is alive right now.
        unsafe { Arc::increment_strong_count(*pointer) };

        RawWaker::new(pointer as *const (), create_vtable::<A>())
    }

    /// Invoke `SharedArcContent::wake_index` with the index in the redirect slice where this pointer points to.
    /// The pointer must satisfy the safety constraints listed in the code comments above.
    unsafe fn wake_by_ref<A: SharedArcContent>(pointer: *const ()) {
        // Retype the type-erased pointer.
        let pointer = pointer as *const *const A;

        // SAFETY: we are already requiring `pointer` to point to a slot in the redirect array.
        let raw: *const A = unsafe { *pointer };
        // SAFETY: we are already requiring the pointer in the redirect array slot to be obtained from `Arc::into_raw`.
        let arc_content: &A = unsafe { &*raw };

        let slice_start = arc_content.get_redirect_slice().as_ptr();

        // We'll switch to [`sub_ptr`](https://github.com/rust-lang/rust/issues/95892) once that's stable.
        let index = unsafe { pointer.offset_from(slice_start) } as usize;

        arc_content.wake_index(index);
    }

    /// Drop the waker (and drop the shared Arc if other Wakers and the combinator have all been dropped).
    /// The pointer must satisfy the safety constraints listed in the code comments above.
    unsafe fn drop_waker<A: SharedArcContent>(pointer: *const ()) {
        // Retype the type-erased pointer.
        let pointer = pointer as *const *const A;

        // SAFETY: we are already requiring `pointer` to point to a slot in the redirect array.
        let raw: *const A = unsafe { *pointer };
        // SAFETY: we are already requiring the pointer in the redirect array slot to be obtained from `Arc::into_raw`.
        unsafe { Arc::decrement_strong_count(raw) };
    }

    /// Like `wake_by_ref` but consumes the Waker.
    /// The pointer must satisfy the safety constraints listed in the code comments above.
    unsafe fn wake<A: SharedArcContent>(pointer: *const ()) {
        // SAFETY: we are already requiring the constraints of `wake_by_ref` and `drop_waker`.
        unsafe {
            wake_by_ref::<A>(pointer);
            drop_waker::<A>(pointer);
        }
    }

    fn create_vtable<A: SharedArcContent>() -> &'static RawWakerVTable {
        &RawWakerVTable::new(
            clone_waker::<A>,
            wake::<A>,
            wake_by_ref::<A>,
            drop_waker::<A>,
        )
    }

    let redirect_slice: &[*const A] = arc.get_redirect_slice();

    debug_assert!(redirect_slice.len() > index);

    // SAFETY: we are already requiring that index be in bound of the slice.
    let pointer: *const *const A = unsafe { redirect_slice.as_ptr().add(index) };
    // Type-erase the pointer because the Waker methods expect so.
    let pointer = pointer as *const ();

    // We want to transfer management of the one strong count associated with `arc` to the Waker we're creating.
    // That count should only be decremented when the Waker is dropped (by `drop_waker`).
    core::mem::forget(arc);

    // SAFETY: All our vtable functions adhere to the RawWakerVTable contract,
    // and we are already requiring that `pointer` is what our functions expect.
    unsafe { Waker::from_raw(RawWaker::new(pointer, create_vtable::<A>())) }
}
