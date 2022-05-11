use std::{cell::RefCell, collections::BTreeSet};

use slab::Slab;

use crate::lifetimed::Borrowed;

use super::{Pushable, Signal};

pub struct Source<T> {
	inner: RefCell<Inner<T>>,
}

struct Inner<T> {
	data: T,
	listeners: BTreeSet<*const dyn Pushable<Borrowed<T>>>,
}

impl<T> Source<T> {
	fn new(data: T) -> Self {
		Self {
			inner: RefCell::new(Inner {
				data,
				listeners: BTreeSet::new(),
			}),
		}
	}
	fn signal<'s>(&'s self) -> SignalSource<'s, T> {
		SignalSource { source: self }
	}
	fn visit_mut<'s, F, R>(&'s self, visitor: F) -> R
	where
		F: for<'x> FnOnce(&'x mut T) -> R,
	{
		let mut inner = self.inner.borrow_mut();
		let ret = visitor(&mut inner.data);
		for listener in inner.listeners.iter() {
			let listener = unsafe { &**listener };
			listener.push(&inner.data);
		}
		ret
	}
	fn set<'s>(&'s self, new_data: T) -> T {
		self.visit_mut(|old| std::mem::replace(old, new_data))
	}
}

pub struct SignalSource<'p, T> {
	source: &'p Source<T>,
}

unsafe impl<'p, T> Signal<Borrowed<T>> for SignalSource<'p, T> {
	fn add_listener<'s>(&'s self, listener: *const dyn Pushable<Borrowed<T>>) {
		self.source.inner.borrow_mut().listeners.insert(listener);
	}
	fn remove_listener<'s>(&'s self, listener: *const dyn Pushable<Borrowed<T>>) {
		self.source.inner.borrow_mut().listeners.remove(&listener);
	}
}
