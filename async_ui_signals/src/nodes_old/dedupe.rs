use std::{cell::RefCell, marker::PhantomPinned};

use crate::{Listenable, Pushable};

use super::ManagedParent;

pub struct SignalDedupe<'p, C: PartialEq + 'p> {
	parent: ManagedParent<'p, Self>,
	listener: RefCell<Option<*const dyn for<'x> Pushable<&'x C>>>,
	cache: RefCell<Option<C>>,
}

impl<'p, C: PartialEq + 'p> SignalDedupe<'p, C> {
	pub fn new(
		parent: &'p (dyn Listenable<Self> + 'p),
	) -> Self {
		Self {
			parent: ManagedParent::new(parent),
			listener: Default::default(),
			cache: Default::default(),
		}
	}
}

impl<'p, C: PartialEq + 'p> Pushable<C> for SignalDedupe<'p, C> {
	fn push<'s>(&'s self, input: C) {
		let mut borrow = self.cache.borrow_mut();
		let changed = if let Some(old) = borrow.as_ref() {
			&input != old
		} else {
			true
		};
		if changed {
			if let Some(listener) = self.listener.borrow().as_ref() {
				let listener = unsafe { &**listener };
				listener.push(&input);
			}
		}
		*borrow = Some(input);
	}
}

impl<'v, 'p, C: PartialEq + 'p, V: for<'x> Pushable<&'x C> + 'v> Listenable<V>
	for SignalDedupe<'p, C>
{
	unsafe fn add_listener<'s, 'z>(&'s self, listener: &'z V) -> usize
	where
		Self: 'z,
	{
		let coerced: *const (dyn for<'x> Pushable<&'x C> + 'v) = listener;
		let transmuted: *const (dyn for<'x> Pushable<&'x C> + 'static) =
			unsafe { std::mem::transmute(coerced) };
		*self.listener.borrow_mut() = Some(transmuted);
		unsafe{self.parent.enable(self)};
		0
	}
	unsafe fn remove_listener<'s, 'z>(&'s self, _key:usize) {
		*self.listener.borrow_mut() = None;
		unsafe {self.parent.disable()};
	}
}

// pub trait Dedupable<'p, C: PartialEq + 'p>: Sized + Listenable<SignalDedupe<'p, C>> {
// 	fn cache(&'p mut self) -> SignalDedupe<'p, C> {
// 		SignalDedupe::new(self)
// 	}
// }
// impl<'p, C: PartialEq + 'p, S: Sized + Listenable<SignalDedupe<'p, C>>> Dedupable<'p, C> for S {}
