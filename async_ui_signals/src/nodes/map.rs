use std::{marker::{PhantomPinned}, cell::RefCell};

use crate::{Listenable, Pushable, mapper::Mapper};



pub struct SignalMap<'p, M>
where
	M: Mapper + 'p,
{
	
	parent: &'p (dyn Listenable<Self> + 'p),
	listener: RefCell<Option<*const dyn for<'x> Pushable<M::Output<'x>>>>,
	mapper: M,
	_pin: PhantomPinned
}

impl<'p, M> SignalMap<'p, M>
where
	M: Mapper + 'p,
{
	pub fn new<P>(mapper: M, parent: &'p P) -> Self
	where
		P: Listenable<Self> + 'p,
	{
		Self {
			parent,
			listener: Default::default(),
			mapper,
			_pin: PhantomPinned
		}
	}
}

impl<'v, 'p, M, V> Listenable<V> for SignalMap<'p, M>
where
	M: Mapper + 'p,
	V: for<'x> Pushable<M::Output<'x>> + 'v,
{
	unsafe fn add_listener<'s, 'z>(&'s self, listener: &'z V) -> usize
	where
		Self: 'z,
	{
		let coerced: *const (dyn for<'x> Pushable<M::Output<'x>> + 'v) = listener;
		let transmuted: *const (dyn for<'x> Pushable<M::Output<'x>> + 'static) =
			unsafe { std::mem::transmute(coerced) };
		*self.listener.borrow_mut() = Some(transmuted);
		0
	}
	unsafe fn remove_listener<'s, 'z>(&'s self, _key:usize) {
		*self.listener.borrow_mut() = None;
	}
}

impl<'i, 'p, M> Pushable<M::Input<'i>> for SignalMap<'p, M>
where
	M: Mapper + 'p,
{
	fn push<'s>(&'s self, input: M::Input<'i>) {
		if let Some(listener) = self.listener.borrow().as_ref() {
			let output = self.mapper.map(input);
			let listener = unsafe { &**listener };
			listener.push(output);
		}
	}
	unsafe fn add_to_parent(&self) {
		unsafe { self.parent.add_listener(self) };
	}
}

// pub trait Mappable<'p, M: Mapper + 'p>: Sized + Listenable<SignalMap<'p, M>> {
// 	fn map(&'p mut self, mapper: M) -> SignalMap<'p, M> {
// 		SignalMap::new(mapper, self)
// 	}
// }
// impl<'p, M: Mapper + 'p, S: Sized + Listenable<SignalMap<'p, M>>> Mappable<'p, M> for S {}
