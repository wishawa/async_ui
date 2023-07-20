use super::super::common::{CombinatorBehaviorVec, CombinatorVec};
use super::{TryJoin as TryJoinTrait, TryJoinBehavior};

use core::future::Future;
use core::ops::ControlFlow;
use std::vec::Vec;

/// Wait for all futures to complete successfully, or abort early on error.
///
/// This `struct` is created by the [`try_join`] method on the [`TryJoin`] trait. See
/// its documentation for more.
///
/// [`try_join`]: crate::future::TryJoin::try_join
/// [`TryJoin`]: crate::future::TryJoin
pub type TryJoin<Fut> = CombinatorVec<Fut, TryJoinBehavior>;

impl<T, E, Fut> CombinatorBehaviorVec<Fut> for TryJoinBehavior
where
    Fut: Future<Output = Result<T, E>>,
{
    const PEND_IF_EMPTY: bool = false;

    type Output = Result<Vec<T>, E>;

    type StoredItem = T;

    fn maybe_return(
        _idx: usize,
        res: <Fut as Future>::Output,
    ) -> ControlFlow<Self::Output, Self::StoredItem> {
        match res {
            Ok(v) => ControlFlow::Continue(v),
            Err(e) => ControlFlow::Break(Err(e)),
        }
    }

    fn when_completed(vec: Vec<Self::StoredItem>) -> Self::Output {
        Ok(vec)
    }
}

impl<T, E, Fut> TryJoinTrait for Vec<Fut>
where
    Fut: Future<Output = Result<T, E>>,
{
    type Ok = Vec<T>;
    type Error = E;
    type Future = TryJoin<Fut>;

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
            let res: io::Result<_> = vec![future::ready(Ok("hello")), future::ready(Ok("world"))]
                .try_join()
                .await;
            assert_eq!(res.unwrap(), vec!["hello", "world"]);
        })
    }

    #[test]
    fn one_err() {
        crate::combinators::block_for_testing(async {
            let err = Error::new(ErrorKind::Other, "oh no");
            let res: io::Result<_> = vec![future::ready(Ok("hello")), future::ready(Err(err))]
                .try_join()
                .await;
            assert_eq!(res.unwrap_err().to_string(), String::from("oh no"));
        });
    }
}
