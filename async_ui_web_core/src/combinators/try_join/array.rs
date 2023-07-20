use super::super::common::{CombinatorArray, CombinatorBehaviorArray};
use super::{TryJoin as TryJoinTrait, TryJoinBehavior};

use core::future::Future;
use core::ops::ControlFlow;

/// Wait for all futures to complete successfully, or abort early on error.
///
/// This `struct` is created by the [`try_join`] method on the [`TryJoin`] trait. See
/// its documentation for more.
///
/// [`try_join`]: crate::future::TryJoin::try_join
/// [`TryJoin`]: crate::future::TryJoin
pub type TryJoin<Fut, const N: usize> = CombinatorArray<Fut, TryJoinBehavior, N>;

impl<T, E, Fut, const N: usize> CombinatorBehaviorArray<Fut, N> for TryJoinBehavior
where
    Fut: Future<Output = Result<T, E>>,
{
    const PEND_IF_EMPTY: bool = false;

    type Output = Result<[T; N], E>;

    type StoredItem = T;

    fn maybe_return(
        _idx: usize,
        res: <Fut as Future>::Output,
    ) -> ControlFlow<Self::Output, Self::StoredItem> {
        match res {
            // Got an Ok result. Keep it.
            Ok(v) => ControlFlow::Continue(v),
            // An error happended. Break now.
            Err(e) => ControlFlow::Break(Err(e)),
        }
    }

    fn when_completed(arr: [Self::StoredItem; N]) -> Self::Output {
        Ok(arr)
    }
}

impl<T, E, Fut, const N: usize> TryJoinTrait for [Fut; N]
where
    Fut: Future<Output = Result<T, E>>,
{
    type Ok = [T; N];
    type Error = E;
    type Future = TryJoin<Fut, N>;

    fn try_join(self) -> Self::Future {
        TryJoin::new(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::future;
    use std::io::{self, Error, ErrorKind};

    #[test]
    fn all_ok() {
        crate::combinators::block_for_testing(async {
            let res: io::Result<_> = [future::ready(Ok("hello")), future::ready(Ok("world"))]
                .try_join()
                .await;
            assert_eq!(res.unwrap(), ["hello", "world"]);
        })
    }

    #[test]
    fn one_err() {
        crate::combinators::block_for_testing(async {
            let err = Error::new(ErrorKind::Other, "oh no");
            let res: io::Result<_> = [future::ready(Ok("hello")), future::ready(Err(err))]
                .try_join()
                .await;
            assert_eq!(res.unwrap_err().to_string(), String::from("oh no"));
        });
    }
}
