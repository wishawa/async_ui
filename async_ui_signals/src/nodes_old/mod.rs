pub mod cache;
pub mod dedupe;
pub mod dedupe_hash;
pub mod for_each;
pub mod map;
pub mod source;
pub mod wrap;
// pub use cache::*;
// pub use dedupe::*;
// pub use dedupe_hash::*;
// pub use map::*;
// pub use source::*;
// pub use for_each::*;

use std::cell::Cell;

use crate::Listenable;

struct ManagedParent<'p, T>
where T: ?Sized
{
	parent: &'p dyn Listenable<T>,
	key: Cell<Option<usize>>,
}

impl<'p, T> ManagedParent<'p, T>
where T: ?Sized
{
	fn new(parent: &'p dyn Listenable<T>) -> Self {
		Self {
			parent,
			key: Cell::new(None),
		}
	}
	unsafe fn enable(&self, listener: &T) {
		if self.key.get().is_none() {
			self.key
				.set(Some(unsafe { self.parent.add_listener(listener) }));
		}
	}
	unsafe fn disable(&self) {
		if let Some(key) = self.key.get() {
			unsafe { self.parent.remove_listener(key) };
		}
	}
}

impl<'p, T> Drop for ManagedParent<'p, T>
where T: ?Sized
{
	fn drop(&mut self) {
		unsafe { self.disable() };
	}
}
