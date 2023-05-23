use core::pin::Pin;
use core::slice::SliceIndex;

/// Returns a pinned mutable reference to an element or subslice depending on the
/// type of index (see `get`) or `None` if the index is out of bounds.
// From: https://github.com/rust-lang/rust/pull/78370/files
#[inline]
pub(crate) fn get_pin_mut<T, I>(slice: Pin<&mut [T]>, index: I) -> Option<Pin<&mut I::Output>>
where
    I: SliceIndex<[T]>,
{
    // SAFETY: `get_unchecked_mut` is never used to move the slice inside `self` (`SliceIndex`
    // is sealed and all `SliceIndex::get_mut` implementations never move elements).
    // `x` is guaranteed to be pinned because it comes from `self` which is pinned.
    unsafe {
        slice
            .get_unchecked_mut()
            .get_mut(index)
            .map(|x| Pin::new_unchecked(x))
    }
}

// NOTE: If this is implemented through the trait, this will work on both vecs and
// slices.
//
// From: https://github.com/rust-lang/rust/pull/78370/files
pub(crate) fn get_pin_mut_from_vec<T, I>(
    slice: Pin<&mut Vec<T>>,
    index: I,
) -> Option<Pin<&mut I::Output>>
where
    I: SliceIndex<[T]>,
{
    // SAFETY: `get_unchecked_mut` is never used to move the slice inside `self` (`SliceIndex`
    // is sealed and all `SliceIndex::get_mut` implementations never move elements).
    // `x` is guaranteed to be pinned because it comes from `self` which is pinned.
    unsafe {
        slice
            .get_unchecked_mut()
            .get_mut(index)
            .map(|x| Pin::new_unchecked(x))
    }
}
