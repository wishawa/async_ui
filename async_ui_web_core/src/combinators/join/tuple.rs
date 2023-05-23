use super::super::common::{CombineTuple, TupleMaybeReturn, TupleWhenCompleted};
use super::{Join as JoinTrait, JoinBehavior};

use core::future::Future;
use core::marker::PhantomData;
use core::ops::ControlFlow;

impl<T, O> TupleMaybeReturn<T, O> for JoinBehavior {
    type StoredItem = T;
    fn maybe_return(_: usize, res: T) -> ControlFlow<O, Self::StoredItem> {
        ControlFlow::Continue(res)
    }
}
impl<O> TupleWhenCompleted<O, O> for JoinBehavior {
    fn when_completed(stored_items: O) -> O {
        stored_items
    }
}

macro_rules! impl_join_tuple {
    ($($F:ident)+) => {
        impl<$($F),+> JoinTrait for ($($F,)+)
        where $(
            $F: Future,
        )+ {
            type Output = ($($F::Output,)+);
            type Future = <(($($F,)+), JoinBehavior, PhantomData<($($F::Output,)+)>) as CombineTuple>::Combined;
            fn join(self) -> Self::Future {
                (
                    self,
                    JoinBehavior,
                    PhantomData
                ).combine()
            }
        }
    };
}

impl JoinTrait for () {
    type Output = ();
    type Future = core::future::Ready<()>;

    fn join(self) -> Self::Future {
        core::future::ready(())
    }
}

impl_join_tuple! { A0 }
impl_join_tuple! { A0 A1 }
impl_join_tuple! { A0 A1 A2 }
impl_join_tuple! { A0 A1 A2 A3 }
impl_join_tuple! { A0 A1 A2 A3 A4 }
impl_join_tuple! { A0 A1 A2 A3 A4 A5 }
impl_join_tuple! { A0 A1 A2 A3 A4 A5 A6 }
impl_join_tuple! { A0 A1 A2 A3 A4 A5 A6 A7 }
impl_join_tuple! { A0 A1 A2 A3 A4 A5 A6 A7 A8 }
impl_join_tuple! { A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 }
impl_join_tuple! { A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 }
impl_join_tuple! { A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 }

#[cfg(test)]
mod test {
    use super::*;
    use std::future;

    #[test]
    #[allow(clippy::unit_cmp)]
    fn join_0() {
        crate::combinators::block_for_testing(async {
            assert_eq!(().join().await, ());
        });
    }

    #[test]
    fn join_1() {
        crate::combinators::block_for_testing(async {
            let a = future::ready("hello");
            assert_eq!((a,).join().await, ("hello",));
        });
    }

    #[test]
    fn join_2() {
        crate::combinators::block_for_testing(async {
            let a = future::ready("hello");
            let b = future::ready(12);
            assert_eq!((a, b).join().await, ("hello", 12));
        });
    }

    #[test]
    fn join_3() {
        crate::combinators::block_for_testing(async {
            let a = future::ready("hello");
            let b = future::ready("world");
            let c = future::ready(12);
            assert_eq!((a, b, c).join().await, ("hello", "world", 12));
        });
    }
}
