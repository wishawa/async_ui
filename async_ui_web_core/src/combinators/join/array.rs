use super::super::common::{CombinatorArray, CombinatorBehaviorArray};
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
pub type Join<Fut, const N: usize> = CombinatorArray<Fut, JoinBehavior, N>;

impl<Fut, const N: usize> CombinatorBehaviorArray<Fut, N> for JoinBehavior
where
    Fut: Future,
{
    const PEND_IF_EMPTY: bool = false;

    type Output = [Fut::Output; N];

    type StoredItem = Fut::Output;

    fn maybe_return(
        _idx: usize,
        res: <Fut as Future>::Output,
    ) -> ControlFlow<Self::Output, Self::StoredItem> {
        // Continue with other subfutures
        ControlFlow::Continue(res)
    }

    fn when_completed(arr: [Self::StoredItem; N]) -> Self::Output {
        arr
    }
}

impl<Fut: Future, const N: usize> JoinTrait for [Fut; N] {
    type Output = [Fut::Output; N];
    type Future = Join<Fut, N>;

    #[inline]
    fn join(self) -> Self::Future {
        Join::new(self)
    }
}

#[cfg(test)]
mod test {
    use futures_lite::future::yield_now;

    use super::*;

    use std::cell::RefCell;
    use std::future;

    #[test]
    fn smoke() {
        crate::combinators::block_for_testing(async {
            let fut = [future::ready("hello"), future::ready("world")].join();
            assert_eq!(fut.await, ["hello", "world"]);
        });
    }

    #[test]
    fn poll_order() {
        let polled = RefCell::new(Vec::new());
        async fn record_poll(id: char, times: usize, target: &RefCell<Vec<char>>) {
            for _ in 0..times {
                target.borrow_mut().push(id);
                yield_now().await;
            }
            target.borrow_mut().push(id);
        }
        crate::combinators::block_for_testing(
            [
                record_poll('a', 0, &polled),
                record_poll('b', 1, &polled),
                record_poll('c', 0, &polled),
            ]
            .join(),
        );
        assert_eq!(&**polled.borrow(), ['a', 'b', 'c', 'b']);

        polled.borrow_mut().clear();
        crate::combinators::block_for_testing(
            [
                record_poll('a', 2, &polled),
                record_poll('b', 3, &polled),
                record_poll('c', 1, &polled),
                record_poll('d', 0, &polled),
            ]
            .join(),
        );
        assert_eq!(
            &**polled.borrow(),
            ['a', 'b', 'c', 'd', 'a', 'b', 'c', 'a', 'b', 'b']
        );
    }
}
