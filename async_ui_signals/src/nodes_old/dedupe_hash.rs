use std::{
	cell::{Cell, RefCell},
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
	marker::PhantomPinned,
};

use crate::{Listenable, Pushable};

use super::ManagedParent;

pub struct SignalDedupeHash<'p, C: Hash + 'p> {
	parent: ManagedParent<'p, Self>,
	listener: RefCell<Option<*const dyn for<'x> Pushable<&'x C>>>,
	cache: Cell<Option<u64>>,
}

impl<'p, C: Hash + 'p> SignalDedupeHash<'p, C> {
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

impl<'p, C: Hash + 'p> Pushable<C> for SignalDedupeHash<'p, C> {
	fn push<'s>(&'s self, input: C) {
		let mut hasher = DefaultHasher::new();
		input.hash(&mut hasher);
		let hash = hasher.finish();

		let changed = if let Some(old) = self.cache.get() {
			old != hash
		} else {
			true
		};
		if changed {
			if let Some(listener) = self.listener.borrow().as_ref() {
				let listener = unsafe { &**listener };
				listener.push(&input);
			}
		}
		self.cache.set(Some(hash));
	}
}

impl<'v, 'p, C: Hash + 'p, V: for<'x> Pushable<&'x C> + 'v> Listenable<V>
	for SignalDedupeHash<'p, C>
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

// pub trait DedupableHash<'p, C: Hash + 'p>: Sized + Listenable<SignalDedupeHash<'p, C>> {
// 	fn cache(&'p mut self) -> SignalDedupeHash<'p, C> {
// 		SignalDedupeHash::new(self)
// 	}
// }
// impl<'p, C: Hash + 'p, S: Sized + Listenable<SignalDedupeHash<'p, C>>> DedupableHash<'p, C> for S {}

