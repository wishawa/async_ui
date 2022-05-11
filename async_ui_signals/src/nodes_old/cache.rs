use std::{cell::RefCell, marker::PhantomPinned};

use crate::{Listenable, Pushable};

use super::ManagedParent;

pub struct SignalCache<'p, C> {
	parent: ManagedParent<'p, Self>,
	listener: RefCell<Option<*const dyn for<'x> Pushable<&'x C>>>,
	cache: RefCell<Option<C>>,
}

impl<'p, C> SignalCache<'p, C> {
	pub fn new(parent: &'p (dyn Listenable<Self> + 'p)) -> Self {
		Self {
			cache: Default::default(),
			listener: Default::default(),
			parent: ManagedParent::new(parent),
		}
	}
}

impl<'v, 'p, C, V> Listenable<V> for SignalCache<'p, C>
where
	V: for<'x> Pushable<&'x C> + 'v,
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

impl<'p, C> Pushable<C> for SignalCache<'p, C> {
	fn push<'s>(&'s self, input: C) {
		let mut borrow = self.cache.borrow_mut();
		let reference = borrow.insert(input);
		if let Some(listener) = self.listener.borrow().as_ref() {
			let listener = unsafe { &**listener };
			listener.push(reference);
		}
	}
}

// pub trait Cachable<'p, C: 'p>: Sized + Listenable<SignalCache<'p, C>> {
// 	fn cache(&'p mut self) -> SignalCache<'p, C> {
// 		SignalCache::new(self)
// 	}
// }
// impl<'p, C: 'p, S: Sized + Listenable<SignalCache<'p, C>>> Cachable<'p, C> for S {}
