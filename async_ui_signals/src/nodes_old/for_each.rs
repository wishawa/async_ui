use std::{marker::PhantomPinned, future::Future, task::Poll};

use async_ui_core::drop_check::check_drop_scope;

use crate::{mapper::Mapper, Listenable, Pushable};

use super::ManagedParent;

pub struct SignalForEach<'p, M>
where
	M: Mapper + 'p,
{
	parent: ManagedParent<'p, Self>,
	func: M,
	_pin: PhantomPinned,
}

impl<'p, M> SignalForEach<'p, M>
where
	M: Mapper + 'p,
{
	pub fn new(parent: &'p (dyn Listenable<Self> + 'p), func: M) -> Self {
		Self { parent: ManagedParent::new(parent), func, _pin: PhantomPinned }
	}
}

impl<'p, 'v, M> Pushable<M::Input<'v>> for SignalForEach<'p, M>
where
	M: Mapper<Output<'v> = ()> + 'p,
{
	fn push<'s>(&'s self, input: M::Input<'v>) {
		self.func.map(input);
	}
}

impl<'p, M> Future for SignalForEach<'p, M>
where
	M: Mapper + 'p
{
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        check_drop_scope(self.as_ref().get_ref() as *const Self as *const ());
		unsafe {self.parent.enable(self.as_ref().get_ref())};
		Poll::Pending
    }
}

// pub trait ForEachable<'p, M: Mapper + 'p>: Sized + Listenable<SignalForEach<'p, M>> {
// 	fn for_each(&'p mut self, func: M) -> SignalForEach<'p, M> {
// 		SignalForEach::new(self, func)
// 	}
// }
// impl<'p, M: Mapper + 'p, S: Sized + Listenable<SignalForEach<'p, M>>> ForEachable<'p, M> for S {}
