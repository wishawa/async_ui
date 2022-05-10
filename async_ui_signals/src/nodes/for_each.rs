use std::marker::PhantomPinned;

use crate::{mapper::Mapper, Listenable, Pushable};

pub struct SignalForEach<'p, M>
where
	M: Mapper + 'p,
{
	parent: &'p (dyn Listenable<Self> + 'p),
	func: M,
	_pin: PhantomPinned,
}

impl<'p, M> SignalForEach<'p, M>
where
	M: Mapper + 'p,
{
	pub fn new(parent: &'p (dyn Listenable<Self> + 'p), func: M) -> Self {
		Self { parent, func, _pin: PhantomPinned }
	}
}

impl<'p, 'v, M> Pushable<M::Input<'v>> for SignalForEach<'p, M>
where
	M: Mapper<Output<'v> = ()> + 'p,
{
	fn push<'s>(&'s self, input: M::Input<'v>) {
		self.func.map(input);
	}
	unsafe fn add_to_parent(&self) {
		unsafe { self.parent.add_listener(self) };
	}
}

// pub trait ForEachable<'p, M: Mapper + 'p>: Sized + Listenable<SignalForEach<'p, M>> {
// 	fn for_each(&'p mut self, func: M) -> SignalForEach<'p, M> {
// 		SignalForEach::new(self, func)
// 	}
// }
// impl<'p, M: Mapper + 'p, S: Sized + Listenable<SignalForEach<'p, M>>> ForEachable<'p, M> for S {}
