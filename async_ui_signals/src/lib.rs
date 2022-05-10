#![feature(generic_associated_types)]
#![forbid(unsafe_op_in_unsafe_fn)]

mod sub;
pub mod nodes;
pub mod mapper;

pub use futures::pin_mut;

use std::{
	cell::RefCell,
	marker::{PhantomData},
	ops::{Index, Range},
	pin::Pin,
};

// use crate::{nodes::{SignalSource, SignalMap, Mappable, Cachable}, mapper::Mapper};

pub trait Listenable<V>
where
	V: ?Sized,
{
	unsafe fn add_listener<'s, 'z>(&'s self, listener: &'z V) -> usize
	where
		Self: 'z;
	unsafe fn remove_listener<'s, 'z>(&'s self, _key:usize);
}

pub trait Pushable<I> {
	fn push<'s>(&'s self, input: I);
	unsafe fn add_to_parent(&self);
	unsafe fn mount(&self) {
		async_ui_core::drop_check::check_drop_scope(self as *const Self as *const ());
		unsafe { self.add_to_parent() };
	}
}

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

// 	let mut mapped = source.map(mapper);

// 	let mapper2: SliceMapper<str> = SliceMapper(1..3, PhantomData);
// 	let mapped2 = mapped.map(mapper2);

// }

// #[cfg(test)]
// mod tests {

// }