use super::super::common::{CombineTuple, TupleMaybeReturn, TupleWhenCompleted};
use super::{Race as RaceTrait, RaceBehavior};

use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::ops::ControlFlow;

impl<T> TupleMaybeReturn<T, T> for RaceBehavior {
    // We early return as soon as any subfuture finishes.
    // Results from subfutures are never stored.
    type StoredItem = Infallible;
    fn maybe_return(_: usize, res: T) -> ControlFlow<T, Self::StoredItem> {
        ControlFlow::Break(res)
    }
}
impl<S, O> TupleWhenCompleted<S, O> for RaceBehavior {
    // We always early return, so we should never get here.
    fn when_completed(_: S) -> O {
        unreachable!() // should have early returned
    }
}

macro_rules! impl_race_tuple {
    ($($F:ident)+) => {
        impl<T, $($F),+> RaceTrait for ($($F,)+)
        where $(
            $F: Future<Output = T>,
        )+ {
            type Output = <Self::Future as Future>::Output;
            type Future = <(($($F,)+), RaceBehavior, PhantomData<T>) as CombineTuple>::Combined;
            fn race(self) -> Self::Future {
                (
                    self,
                    RaceBehavior,
                    PhantomData
                ).combine()
            }
        }
    };
}

impl_race_tuple! { A0 }
impl_race_tuple! { A0 A1 }
impl_race_tuple! { A0 A1 A2 }
impl_race_tuple! { A0 A1 A2 A3 }
impl_race_tuple! { A0 A1 A2 A3 A4 }
impl_race_tuple! { A0 A1 A2 A3 A4 A5 }
impl_race_tuple! { A0 A1 A2 A3 A4 A5 A6 }
impl_race_tuple! { A0 A1 A2 A3 A4 A5 A6 A7 }
impl_race_tuple! { A0 A1 A2 A3 A4 A5 A6 A7 A8 }
impl_race_tuple! { A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 }
impl_race_tuple! { A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 }
impl_race_tuple! { A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 }

#[cfg(test)]
mod test {
    use super::*;
    use std::future;

    #[test]
    fn race_1() {
        crate::combinators::block_for_testing(async {
            let a = future::ready("world");
            assert_eq!((a,).race().await, "world");
        });
    }

    #[test]
    fn race_2() {
        crate::combinators::block_for_testing(async {
            let a = future::pending();
            let b = future::ready("world");
            assert_eq!((a, b).race().await, "world");
        });
    }

    #[test]
    fn race_3() {
        crate::combinators::block_for_testing(async {
            let a = future::pending();
            let b = future::ready("hello");
            let c = future::ready("world");
            let result = (a, b, c).race().await;
            assert!(matches!(result, "hello" | "world"));
        });
    }
}
