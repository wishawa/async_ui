use super::super::common::{CombineTuple, TupleMaybeReturn, TupleWhenCompleted};
use super::{RaceOk as RaceOkTrait, RaceOkBehavior};

use core::future::Future;
use core::marker::PhantomData;
use core::ops::ControlFlow;

impl<T, E, AggE> TupleMaybeReturn<Result<T, E>, Result<T, AggE>> for RaceOkBehavior {
    type StoredItem = E;
    fn maybe_return(_: usize, res: Result<T, E>) -> ControlFlow<Result<T, AggE>, Self::StoredItem> {
        match res {
            Ok(t) => ControlFlow::Break(Ok(t)),
            Err(e) => ControlFlow::Continue(e),
        }
    }
}
impl<T, AggE> TupleWhenCompleted<AggE, Result<T, AggE>> for RaceOkBehavior {
    // If we get here, it must have been that none of the subfutures early returned.
    // This means all of them failed. In this case we returned a tuple with the errors we kept.
    fn when_completed(errors: AggE) -> Result<T, AggE> {
        Err(errors)
    }
}

macro_rules! impl_race_ok_tuple {
    ($(($F:ident $E:ident))+) => {
        impl<T, $($E,)+ $($F),+> RaceOkTrait for ($($F,)+)
        where $(
            $F: Future<Output = Result<T, $E>>,
        )+ {
            type Ok = T;
            type Error = ($($E, )+);
            type Future = <(($($F,)+), RaceOkBehavior, PhantomData<Result<T, ($($E,)+)>>) as CombineTuple>::Combined;
            fn race_ok(self) -> Self::Future {
                (
                    self,
                    RaceOkBehavior,
                    PhantomData
                ).combine()
            }
        }
    };
}

impl_race_ok_tuple! { (A0 E0) }
impl_race_ok_tuple! { (A0 E0) (A1 E1) }
impl_race_ok_tuple! { (A0 E0) (A1 E1) (A2 E2) }
impl_race_ok_tuple! { (A0 E0) (A1 E1) (A2 E2) (A3 E3) }
impl_race_ok_tuple! { (A0 E0) (A1 E1) (A2 E2) (A3 E3) (A4 E4) }
impl_race_ok_tuple! { (A0 E0) (A1 E1) (A2 E2) (A3 E3) (A4 E4) (A5 E5) }
impl_race_ok_tuple! { (A0 E0) (A1 E1) (A2 E2) (A3 E3) (A4 E4) (A5 E5) (A6 E6) }
impl_race_ok_tuple! { (A0 E0) (A1 E1) (A2 E2) (A3 E3) (A4 E4) (A5 E5) (A6 E6) (A7 E7) }
impl_race_ok_tuple! { (A0 E0) (A1 E1) (A2 E2) (A3 E3) (A4 E4) (A5 E5) (A6 E6) (A7 E7) (A8 E8) }
impl_race_ok_tuple! { (A0 E0) (A1 E1) (A2 E2) (A3 E3) (A4 E4) (A5 E5) (A6 E6) (A7 E7) (A8 E8) (A9 E9) }
impl_race_ok_tuple! { (A0 E0) (A1 E1) (A2 E2) (A3 E3) (A4 E4) (A5 E5) (A6 E6) (A7 E7) (A8 E8) (A9 E9) (A10 E10) }

#[cfg(test)]
mod test {
    use super::*;
    use core::future;
    use std::error::Error;

    type DynError = Box<dyn Error>;

    #[test]
    fn race_ok_1() {
        crate::combinators::block_for_testing(async {
            let a = async { Ok::<_, DynError>("world") };
            let res = (a,).race_ok().await;
            assert!(matches!(res.unwrap(), "world"));
        });
    }

    #[test]
    fn race_ok_2() {
        crate::combinators::block_for_testing(async {
            let a = future::pending::<Result<&str, ()>>();
            let b = async { Ok::<_, DynError>("world") };
            let res = (a, b).race_ok().await;
            assert!(matches!(res.unwrap(), "world"));
        });
    }

    #[test]
    fn race_ok_3() {
        crate::combinators::block_for_testing(async {
            let a = future::pending::<Result<&str, ()>>();
            let b = async { Ok::<_, DynError>("hello") };
            let c = async { Ok::<_, DynError>("world") };
            let result = (a, b, c).race_ok().await;
            assert!(matches!(result.unwrap(), "hello" | "world"));
        });
    }

    #[test]
    fn race_ok_err() {
        crate::combinators::block_for_testing(async {
            let a = async { Err::<(), _>("hello") };
            let b = async { Err::<(), _>("world") };
            let errors = (a, b).race_ok().await.unwrap_err();
            assert_eq!(errors.0, "hello");
            assert_eq!(errors.1, "world");
        });
    }
}
