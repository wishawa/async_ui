use std::cell::RefCell;

use crate::{Pushable, Listenable};

pub struct SignalSource<T> {
	value: RefCell<T>,
	listener: RefCell<Option<*const dyn for<'x> Pushable<&'x T>>>,
}

impl<T> SignalSource<T> {
	pub fn new(value: T) -> Self {
		Self {
			value: RefCell::new(value),
			listener: Default::default(),
		}
	}
	pub fn visit_mut<F, R>(&self, visitor: F)
	where
		F: FnOnce(&mut T) -> R
	{
		let mut bm = self.value.borrow_mut();
		visitor(&mut *bm);
		if let Some(listener) = self.listener.borrow().as_ref() {
			let listener = unsafe {&**listener};
			listener.push(&*bm);
		}
	}
}

impl<'v, T, V> Listenable<V> for SignalSource<T>
where
	V:  for<'x> Pushable<&'x T> + 'v,
{
	unsafe fn add_listener<'s, 'z>(&'s self, listener: &'z V) -> usize
	where
		Self: 'z,
	{
		let coerced: *const (dyn for<'x> Pushable<&'x T> + 'v) = listener;
		let transmuted: *const (dyn for<'x> Pushable<&'x T> + 'static) =
			unsafe { std::mem::transmute(coerced) };
		*self.listener.borrow_mut() = Some(transmuted);
		0
	}
	unsafe fn remove_listener<'s, 'z>(&'s self, _key:usize) {
		*self.listener.borrow_mut() = None;
	}
}