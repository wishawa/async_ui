use crate::{
	lifetimed::{Borrowed, Lifetimed, Owned},
	signal::Signal,
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

// fn test(sig: &mut dyn Signal<Borrowed<str>>) {
// 	let mut mapped1 = sig.map(|inp, next: PushNext<Borrowed<str>>| {

// 	});
// 	let mut mapped2 = mapped1.map(|inp, next: PushNext<Borrowed<[u8]>>| {

// 	});
// 	let mut mapped3 = mapped2.map(|inp, next: PushNext<Owned<i32>>| {

// 	});
// 	let mut mapped4 = mapped3.map(|inp, next: PushNext<Owned<()>>| {

// 	});
// }
