#![feature(generic_associated_types)]

use std::{
	marker::PhantomData,
	ops::{Index, Range},
};

use async_ui_signals::{
	nodes::{Mapper, SignalCached, SignalCell, SignalMap},
	Visitable,
};

fn main() {
	let s = String::from("hello world");
	let source = SignalCell::new(s);

	struct SliceMapper<T: ?Sized + Index<Range<usize>>>(Range<usize>, PhantomData<T>);
	impl<T: ?Sized + Index<Range<usize>>> Mapper for SliceMapper<T> {
		type Input<'i> = &'i T where Self: 'i;
		type Output<'o> = &'o T::Output where Self: 'o;
		fn map<'x>(&self, input: Self::Input<'x>) -> Self::Output<'x> {
			&input[self.0.clone()]
		}
	}

	let mapper = SliceMapper(1..5, PhantomData);
	let mapped = SignalMap::new(&source as _, mapper);
	source.visit(&mut |s: &String| {
		assert_eq!(s, "hello world");
	});
	mapped.visit(&mut |s: &str| {
		assert_eq!(s, "ello");
	});
	let s = [0, 1, 2, 3, 4, 5, 6, 7, 8];
	let source = SignalCell::new(s);
	let mapper = SliceMapper(1..5, PhantomData);
	let mapped = SignalMap::new(&source as _, mapper);
	struct ToOwnedMapper<T: ?Sized + ToOwned>(PhantomData<T>);
	impl<T: ?Sized + ToOwned> Mapper for ToOwnedMapper<T> {
		type Input<'i> = &'i T where Self: 'i;
		type Output<'o> = T::Owned where Self: 'o;
		fn map<'m, 's>(&'s self, input: Self::Input<'m>) -> Self::Output<'m>
		where
			Self: 'm,
		{
			input.to_owned()
		}
	}
	let to_owned = SignalMap::new(&mapped as _, ToOwnedMapper(PhantomData));
	let cached = SignalCached::new(&to_owned as _);
	source.visit(&mut |s: &[i32; 9]| {
		assert_eq!(s, &[0, 1, 2, 3, 4, 5, 6, 7, 8]);
	});
	mapped.visit(&mut |s: &[i32]| {
		assert_eq!(s, &[1, 2, 3, 4]);
	});
	cached.visit(&mut |s: &Vec<i32>| {
		assert_eq!(s, &vec![1, 2, 3, 4]);
	});
}
