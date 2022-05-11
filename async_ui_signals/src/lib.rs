#![feature(generic_associated_types)]
// #![feature(unsize)]
#![forbid(unsafe_op_in_unsafe_fn)]

mod ext;
mod lifetimed;
pub mod mapper;
mod signal;
mod sub;

pub use futures::pin_mut;

use std::{
	cell::RefCell,
	marker::PhantomData,
	ops::{Index, Range},
	pin::Pin,
};

// use crate::{nodes::{SignalSource, SignalMap, Mappable, Cachable}, mapper::Mapper};
// use crate::{nodes::{source::SignalSource, map::SignalMap, wrap::{WrapPushable, BorrowOf}, for_each::SignalForEach}, mapper::Mapper};

fn test() {
	let s = String::from("hello world");
	// let mut source = Source {
	// 	value: RefCell::new(s),
	// 	listener: Default::default(),
	// };
	// let mut mapped1 = source.map(|inp, next: PushNext<Borrow<str>>| {
	// 	next.push(&inp[1..5]);
	// });
	// let mapped2 = mapped1.map(|inp, next: PushNext<Borrow<[u8]>>| {
	// 	next.push(inp.as_bytes());
	// });
	// let mapped1 = Map {
	// 	parent: &source,
	// 	mapper: |inp: &String, next: PushNext<'_, Borrow<str>>| {
	// 		next.push(&inp[1..5]);
	// 	},
	// 	listener: Default::default()
	// };
	// let mapped2 = Map {
	// 	parent: &mapped1,
	// 	mapper: |inp: &str, next: PushNext<'_, Borrow<str>>| {
	// 		next.push(inp);
	// 	},
	// 	listener: Default::default()
	// };
}

// pub trait Listenable<V>
// where
// 	V: ?Sized,
// {
// 	unsafe fn add_listener<'s, 'z>(&'s self, listener: &'z V) -> usize
// 	where
// 		Self: 'z;
// 	unsafe fn remove_listener<'s, 'z>(&'s self, _key:usize);
// }

// pub trait Pushable<I> {
// 	fn push<'s>(&'s self, input: I);
// }

// fn play() {
// 	let s = String::from("hello world");
// 	let mut source = SignalSource::new(s);

// 	struct SliceMapper<T: ?Sized + Index<Range<usize>>>(Range<usize>, PhantomData<T>);
// 	impl<T: ?Sized + Index<Range<usize>>> Mapper for SliceMapper<T> {
// 		type Input<'i> = &'i T where Self: 'i;
// 		type Output<'o> = &'o T::Output where Self: 'o;
// 		fn map<'x>(&self, input: Self::Input<'x>) -> Self::Output<'x> {
// 			&input[self.0.clone()]
// 		}
// 	}

// 	let mapper: SliceMapper<String> = SliceMapper(1..5, PhantomData);

// 	let mut mapped = SignalMap::new(mapper, &source);

// 	let mapper2: SliceMapper<str> = SliceMapper(1..3, PhantomData);
// 	let mapped2 = SignalMap::new(mapper2, &mapped);
// 	// fn take_sig(signal: &(dyn for<'k> Listenable<dyn for<'i> Pushable<&'i str> + 'k> + '_)) {
// 		// let mapper2: SliceMapper<str> = SliceMapper(1..3, PhantomData);
// 		// let mapped2 = SignalMap::new(mapper2, signal);
// 	// }
// 	struct TestMapper;
// 	impl Mapper for TestMapper {
// 		type Input<'i> = &'i str where Self: 'i;
// 		type Output<'o> = () where Self :'o;
// 		fn map<'m, 's>(&'s self, input: Self::Input<'m>) -> Self::Output<'m>
// 		where
// 				Self: 'm {
// 			//println!("{}", input);
// 			panic!("OH NOO");
// 		}
// 	}

// 	// take_sig(&mapped as &(dyn for<'k> Listenable<dyn for<'i> Pushable<&'i str> + 'k> + '_));
// }

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test() {
		// play();
	}
}
