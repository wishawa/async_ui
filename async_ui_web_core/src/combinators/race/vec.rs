use super::super::common::{CombinatorBehaviorVec, CombinatorVec};
use super::{Race as RaceTrait, RaceBehavior};

use core::future::Future;
use core::ops::ControlFlow;

/// Wait for the first future to complete.
///
/// This `struct` is created by the [`race`] method on the [`Race`] trait. See
/// its documentation for more.
///
/// [`race`]: crate::future::Race::race
/// [`Race`]: crate::future::Race
pub type Race<Fut> = CombinatorVec<Fut, RaceBehavior>;

impl<Fut> CombinatorBehaviorVec<Fut> for RaceBehavior
where
    Fut: Future,
{
    const PEND_IF_EMPTY: bool = true;

    type Output = Fut::Output;

    type StoredItem = core::convert::Infallible;

    fn maybe_return(
        _idx: usize,
        res: <Fut as Future>::Output,
    ) -> ControlFlow<Self::Output, Self::StoredItem> {
        ControlFlow::Break(res)
    }

    fn when_completed(_vec: Vec<Self::StoredItem>) -> Self::Output {
        unreachable!()
    }
}

impl<Fut: Future> RaceTrait for Vec<Fut> {
    type Output = Fut::Output;
    type Future = Race<Fut>;

    fn race(self) -> Self::Future {
        Race::new(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::future;

    // NOTE: we should probably poll in random order.
    #[test]
    fn no_fairness() {
        crate::combinators::block_for_testing(async {
            let res = vec![future::ready("hello"), future::ready("world")]
                .race()
                .await;
            assert!(matches!(res, "hello" | "world"));
        });
    }
}
