use std::{cell::RefCell, collections::BTreeSet, rc::{Rc, Weak}, task::{Poll, Context}, pin::Pin};

use futures::Future;

use crate::lifetimed::Borrowed;

use super::{Pushable, Signal, PushMode};

pub struct Source<T> {
	inner: RefCell<Inner<T>>,
}

struct Inner<T> {
	data: T,
	listeners: BTreeSet<*const dyn Pushable<Borrowed<T>>>,
	requesters: BTreeSet<*const dyn Pushable<Borrowed<T>>>,
	scheduled_fire: Rc<RefCell<Option<*const dyn Fire>>>
}

struct Scheduled(Weak<RefCell<Option<*const dyn Fire>>>);

impl Future for Scheduled {
	type Output = ();
	fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
		if let Some(rc) = self.0.upgrade() {
			if let Some(firer) = rc.borrow_mut().take() {
				let firer = unsafe {&*firer};
				firer.fire(PushMode::Requested);
			}
		}
		Poll::Ready(())
	}
}

trait Fire {
	fn fire(&self, mode: PushMode);
}

impl<T> Fire for Source<T> {
	fn fire(&self, mode: PushMode) {
		let inner = self.inner.borrow();
		for listener in inner.listeners.iter() {
			let listener = unsafe {&**listener};
			listener.push(&inner.data, mode);
		}
	}
}

impl<T> Source<T> {
	fn new(data: T) -> Self {
		Self {
			inner: RefCell::new(Inner {
				data,
				listeners: BTreeSet::new(),
				requesters: BTreeSet::new(),
				scheduled_fire: Default::default()
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
		*inner.scheduled_fire.borrow_mut() = None;
		self.fire(PushMode::NotRequested);
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
		let mut inner = self.source.inner.borrow_mut();
		inner.listeners.remove(&listener);
		inner.requesters.remove(&listener);
		if inner.requesters.is_empty() {
			*inner.scheduled_fire.borrow_mut() = None;
		}
	}
	fn request_fire<'s>(&'s self, listener: *const dyn Pushable<Borrowed<T>>) {
		self.source.inner.borrow_mut().requesters.insert(value);
		let ptr: &(dyn Fire + 's) = self.source;
		let ptr: *const (dyn Fire + 's) = ptr;
		let ptr: *const (dyn Fire + 'static) = unsafe {std::mem::transmute(ptr)};
		*self.source.inner.borrow_mut().scheduled_fire.borrow_mut() = Some(ptr);
	}
}
