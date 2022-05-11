use std::marker::PhantomData;

use crate::{Listenable, Pushable};

pub trait Lifetimed {
	type Value<'a> where Self: 'a;
}

pub struct BorrowOf<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> Lifetimed for BorrowOf<T> {
	type Value<'a> = &'a T where Self: 'a;
}

pub struct WrapPushable<'b, S>
where
	S: Lifetimed + 'b,
{
	pushable: &'b (dyn for<'i> Pushable<S::Value<'i>> + 'b),
}

impl<'b, S> WrapPushable<'b, S>
where
	S: Lifetimed + 'b,
{
	pub fn new(pushable: &'b (dyn for<'i> Pushable<S::Value<'i>> + 'b)) -> Self {
		Self { pushable }
	}
}

impl<'b, 'v, S> Pushable<S::Value<'v>> for WrapPushable<'b, S>
where
	S: Lifetimed + 'b
{
	fn push<'s>(&'s self, input: S::Value<'v>) {
		self.pushable.push(input);
	}
}

// pub struct WrapSignal<'p, S>
// where
// 	S: SignalOutput
// {
// 	parent: &'p dyn Listenable<Self>
// }

// impl<'p, 'v, S> Pushable<S::Value<'v>> for WrapSignal<'p, S>
// where
// 	S: SignalOutput
// {
// 	fn push<'s>(&'s self, input: S::Value<'v>) {

// 	}
// }
