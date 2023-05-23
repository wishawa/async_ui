use core::future::Future;

pub(crate) mod array;
pub(crate) mod tuple;
pub(crate) mod vec;

/// Wait for the first successful future to complete.
///
/// Awaits multiple futures simultaneously, returning the output of the first
/// future which completes. If no future completes successfully, returns an
/// aggregate error of all failed futures.
pub trait RaceOk {
    /// The resulting output type.
    type Ok;

    /// The resulting error type.
    type Error;

    /// Which kind of future are we turning this into?
    type Future: Future<Output = Result<Self::Ok, Self::Error>>;

    /// Waits for the first successful future to complete.
    fn race_ok(self) -> Self::Future;
}

#[derive(Debug)]
pub struct RaceOkBehavior;
