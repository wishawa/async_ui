use super::super::common::{CombinatorBehaviorVec, CombinatorVec};
use super::{RaceOk as RaceOkTrait, RaceOkBehavior};

use core::future::Future;
use core::ops::ControlFlow;
use std::vec::Vec;

/// Wait for the first successful future to complete.
///
/// This `struct` is created by the [`race_ok`] method on the [`RaceOk`] trait. See
/// its documentation for more.
///
/// [`race_ok`]: crate::future::RaceOk::race_ok
/// [`RaceOk`]: crate::future::RaceOk
pub type RaceOk<Fut> = CombinatorVec<Fut, RaceOkBehavior>;

impl<T, E, Fut> CombinatorBehaviorVec<Fut> for RaceOkBehavior
where
    Fut: Future<Output = Result<T, E>>,
{
    const PEND_IF_EMPTY: bool = false;

    type Output = Result<T, Vec<E>>;

    type StoredItem = E;

    fn maybe_return(
        _idx: usize,
        res: <Fut as Future>::Output,
    ) -> ControlFlow<Self::Output, Self::StoredItem> {
        match res {
            Ok(v) => ControlFlow::Break(Ok(v)),
            Err(e) => ControlFlow::Continue(e),
        }
    }

    fn when_completed(errors: Vec<Self::StoredItem>) -> Self::Output {
        Err(errors)
    }
}

impl<Fut, T, E> RaceOkTrait for Vec<Fut>
where
    Fut: Future<Output = Result<T, E>>,
{
    type Ok = T;
    type Error = Vec<E>;
    type Future = RaceOk<Fut>;

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
            let res = vec![
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
            let res = vec![future::ready(Ok("hello")), future::ready(Err(err))]
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
            let res = vec![future::ready(Err::<(), _>(err1)), future::ready(Err(err2))]
                .race_ok()
                .await;
            let err = res.unwrap_err();
            assert_eq!(err[0].to_string(), "oops");
            assert_eq!(err[1].to_string(), "oh no");
        });
    }
}
