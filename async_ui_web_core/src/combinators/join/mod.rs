use core::future::Future;

pub(crate) mod array;
pub(crate) mod tuple;
pub(crate) mod vec;

/// Wait for all futures to complete.
///
/// Awaits multiple futures simultaneously, returning the output of the futures
/// once all complete.
pub trait Join {
    /// The resulting output type.
    type Output;

    /// Which kind of future are we turning this into?
    type Future: Future<Output = Self::Output>;

    /// Waits for multiple futures to complete.
    ///
    /// Awaits multiple futures simultaneously, returning the output of the
    /// futures once all complete.
    ///
    /// This function returns a new future which polls all futures concurrently.
    fn join(self) -> Self::Future;
}

#[derive(Debug)]
pub struct JoinBehavior;
