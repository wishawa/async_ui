use super::super::common::{CombinatorBehaviorVec, CombinatorVec};
use super::{Join as JoinTrait, JoinBehavior};

use core::future::Future;
use core::ops::ControlFlow;

/// Waits for two similarly-typed futures to complete.
///
/// This `struct` is created by the [`join`] method on the [`Join`] trait. See
/// its documentation for more.
///
/// [`join`]: crate::future::Join::join
/// [`Join`]: crate::future::Join
pub type Join<Fut> = CombinatorVec<Fut, JoinBehavior>;

impl<Fut> CombinatorBehaviorVec<Fut> for JoinBehavior
where
    Fut: Future,
{
    type Output = Vec<Fut::Output>;

    type StoredItem = Fut::Output;

    fn maybe_return(
        _idx: usize,
        res: <Fut as Future>::Output,
    ) -> ControlFlow<Self::Output, Self::StoredItem> {
        ControlFlow::Continue(res)
    }

    fn when_completed(vec: Vec<Self::StoredItem>) -> Self::Output {
        vec
    }
}

impl<Fut: Future> JoinTrait for Vec<Fut> {
    type Output = Vec<Fut::Output>;
    type Future = Join<Fut>;

    fn join(self) -> Self::Future {
        Join::new(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::future;

    #[test]
    fn smoke() {
        crate::combinators::block_for_testing(async {
            let fut = vec![future::ready("hello"), future::ready("world")].join();
            assert_eq!(fut.await, vec!["hello", "world"]);
        });
    }
}
