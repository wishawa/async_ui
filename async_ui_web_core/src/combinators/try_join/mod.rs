use core::future::Future;

pub(crate) mod array;
pub(crate) mod tuple;
pub(crate) mod vec;

/// Wait for all futures to complete successfully, or abort early on error.
///
/// In the case a future errors, all other futures will be cancelled. If
/// futures have been completed, their results will be discarded.
///
/// If you want to keep partial data in the case of failure, see the `merge`
/// operation.
pub trait TryJoin {
    /// The resulting output type.
    type Ok;

    /// The resulting error type.
    type Error;

    /// Which kind of future are we turning this into?
    type Future: Future<Output = Result<Self::Ok, Self::Error>>;

    /// Waits for multiple futures to complete, either returning when all
    /// futures complete successfully, or return early when any future completes
    /// with an error.
    fn try_join(self) -> Self::Future;
}

#[derive(Debug)]
pub struct TryJoinBehavior;
