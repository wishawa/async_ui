use super::super::common::{CombinatorArray, CombinatorBehaviorArray};
use super::{RaceOk as RaceOkTrait, RaceOkBehavior};

use core::future::Future;
use core::ops::ControlFlow;

/// Wait for the first successful future to complete.
///
/// This `struct` is created by the [`race_ok`] method on the [`RaceOk`] trait. See
/// its documentation for more.
///
/// [`race_ok`]: crate::future::RaceOk::race_ok
/// [`RaceOk`]: crate::future::RaceOk
pub type RaceOk<Fut, const N: usize> = CombinatorArray<Fut, RaceOkBehavior, N>;

impl<T, E, Fut, const N: usize> CombinatorBehaviorArray<Fut, N> for RaceOkBehavior
where
    Fut: Future<Output = Result<T, E>>,
{
    type Output = Result<T, [E; N]>;

    type StoredItem = E;

    fn maybe_return(
        _idx: usize,
        res: <Fut as Future>::Output,
    ) -> ControlFlow<Self::Output, Self::StoredItem> {
        match res {
            // Got an Ok result. Break now.
            Ok(v) => ControlFlow::Break(Ok(v)),
            // Err result. Continue polling other subfutures.
            Err(e) => ControlFlow::Continue(e),
        }
    }

    fn when_completed(errors: [Self::StoredItem; N]) -> Self::Output {
        Err(errors)
    }
}

impl<T, E, Fut, const N: usize> RaceOkTrait for [Fut; N]
where
    Fut: Future<Output = Result<T, E>>,
{
    type Ok = T;
    type Error = [E; N];
    type Future = RaceOk<Fut, N>;

    fn race_ok(self) -> Self::Future {
        RaceOk::new(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::future;
    use std::io::{Error, ErrorKind};

    #[test]
    fn all_ok() {
        crate::combinators::block_for_testing(async {
            let res = [
                future::ready(Ok::<_, ()>("hello")),
                future::ready(Ok("world")),
            ]
            .race_ok()
            .await;
            assert!(res.is_ok());
        })
    }

    #[test]
    fn one_err() {
        crate::combinators::block_for_testing(async {
            let err = Error::new(ErrorKind::Other, "oh no");
            let res = [future::ready(Ok("hello")), future::ready(Err(err))]
                .race_ok()
                .await;
            assert_eq!(res.unwrap(), "hello");
        });
    }

    #[test]
    fn all_err() {
        crate::combinators::block_for_testing(async {
            let err1 = Error::new(ErrorKind::Other, "oops");
            let err2 = Error::new(ErrorKind::Other, "oh no");
            let res = [future::ready(Err::<(), _>(err1)), future::ready(Err(err2))]
                .race_ok()
                .await;
            let err = res.unwrap_err();
            assert_eq!(err[0].to_string(), "oops");
            assert_eq!(err[1].to_string(), "oh no");
        });
    }
}
