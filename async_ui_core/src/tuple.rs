use smallvec::SmallVec;

use crate::{backend::Backend, element::Element};
use std::future::Future;

macro_rules! make_tuples {
	($($id:expr),*) => {
		paste::paste! {
			impl<'f, $([<F $id>]: Future<Output = ()> + 'f),*> TupleOfFutures<'f> for ($([<F $id>],)*) {
				fn internal_convert_to_smallvec_element<B: Backend>(self) -> SmallVec<[Element<'f, B>; 4]> {
					let ($([<v_ $id>],)*) = self;
					smallvec::smallvec![$(
						[<v_ $id>].into()
					),*]
				}
			}
		}
	};
}
make_tuples!(1);
make_tuples!(1, 2);
make_tuples!(1, 2, 3);
make_tuples!(1, 2, 3, 4);
make_tuples!(1, 2, 3, 4, 5);
make_tuples!(1, 2, 3, 4, 5, 6);
make_tuples!(1, 2, 3, 4, 5, 6, 7);
make_tuples!(1, 2, 3, 4, 5, 6, 7, 8);
make_tuples!(1, 2, 3, 4, 5, 6, 7, 8, 9);
make_tuples!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
make_tuples!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
make_tuples!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);

pub trait TupleOfFutures<'f> {
    fn internal_convert_to_smallvec_element<B: Backend>(self) -> SmallVec<[Element<'f, B>; 4]>;
}
