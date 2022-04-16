use crate::{backend::Backend, element::Element};

pub trait TupleInto<T> {
    type Slice: IntoIterator<Item = T>;
    fn convert_to_slice(self) -> Self::Slice;
}

macro_rules! make_tuples {
	($num:expr, $($id:ident),*) => {
		impl<T, $($id: Into<T>),*> TupleInto<T> for ($($id,)*) {
			type Slice = [T; $num];
			#[allow(non_snake_case)]
			fn convert_to_slice(self) -> Self::Slice {
				let ($($id,)*) = self;
				[$(
					$id.into()
				),*]
			}
		}
	};
}
make_tuples!(01, A1);
make_tuples!(02, A1, A2);
make_tuples!(03, A1, A2, A3);
make_tuples!(04, A1, A2, A3, A4);
make_tuples!(05, A1, A2, A3, A4, A5);
make_tuples!(06, A1, A2, A3, A4, A5, A6);
make_tuples!(07, A1, A2, A3, A4, A5, A6, A7);
make_tuples!(08, A1, A2, A3, A4, A5, A6, A7, A8);
make_tuples!(09, A1, A2, A3, A4, A5, A6, A7, A8, A9);
make_tuples!(10, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
make_tuples!(11, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
make_tuples!(12, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);

pub trait TupleOfFutures<'e, B: Backend>: TupleInto<Element<'e, B>> {}
impl<'e, B: Backend, T: TupleInto<Element<'e, B>>> TupleOfFutures<'e, B> for T {}
