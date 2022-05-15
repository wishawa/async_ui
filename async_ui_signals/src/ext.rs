use std::hash::Hash;

use crate::{
	lifetimed::{Borrowed, Lifetimed, Owned},
	signal::{Signal, map::{map, MapTo}, dedupe::{dedupe_by_hash, dedupe}},
};

pub trait UnsizeSignal<L>
where
	L: Lifetimed,
{
	fn coerce<'s>(&'s self) -> &'s (dyn Signal<L> + 's);
}

impl<L, S> UnsizeSignal<L> for S
where
	L: Lifetimed,
	S: Signal<L>,
{
	fn coerce<'s>(&'s self) -> &(dyn Signal<L> + 's) {
		self
	}
}

impl<'b, L> UnsizeSignal<L> for (dyn Signal<L> + 'b)
where
	L: Lifetimed,
{
	fn coerce<'s>(&'s self) -> &'s (dyn Signal<L> + 's) {
		self
	}
}

fn test(sig: &mut dyn Signal<Borrowed<str>>) {
	let mut mapped1 = map(sig, |inp, next: MapTo<Borrowed<str>>| {

	});
	let mut mapped2 = map(&mut mapped1, |inp, next: MapTo<Borrowed<[u8]>>| {

	});
	let mut mapped3 = map(&mut mapped2, |inp, next: MapTo<Owned<i32>>| {

	});
	let mut deduped = dedupe_by_hash::<'_, _, Owned<i32>>(&mut mapped3);
	let mut deduped = dedupe::<'_, _, _, i32>(&mut deduped);
	let mut mapped4 = map(&mut deduped, |inp, next: MapTo<Owned<()>>| {

	});
}
