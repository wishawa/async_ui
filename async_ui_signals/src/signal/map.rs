use std::cell::RefCell;

use crate::{ext::UnsizeSignal, lifetimed::Lifetimed};

use super::{Pushable, Signal, PushMode};

pub struct MapTo<'f, I>
where
	I: Lifetimed,
{
	fire: &'f dyn Pushable<I>,
	mode: PushMode
}
impl<'f, I> MapTo<'f, I>
where
	I: Lifetimed,
{
	pub fn push<'v>(self, value: I::Value<'v>)
	where
		'f: 'v,
	{
		self.fire.push(value, self.mode);
	}
}

pub struct Map<'p, I, M, O>
where
	I: Lifetimed,
	O: Lifetimed,
	M: for<'v> FnMut(I::Value<'v>, MapTo<'v, O>),
{
	parent: &'p (dyn Signal<I> + 'p),
	inner: RefCell<MapInner<M, O>>,
}

struct MapInner<M, O>
where
	O: Lifetimed,
{
	listener: Option<*const dyn Pushable<O>>,
	mapper: M,
	fire_requested: bool,
}

impl<'p, I, M, O> Map<'p, I, M, O>
where
	I: Lifetimed,
	O: Lifetimed,
	M: for<'v> FnMut(I::Value<'v>, MapTo<'v, O>),
{
	unsafe fn transmute_to_dyn(&self) -> *const dyn Pushable<I> {
		let coerced: *const (dyn Pushable<I> + '_) = self;
		let transmuted: *const (dyn Pushable<I> + 'static) =
			unsafe { std::mem::transmute(coerced) };
		transmuted
	}
}



unsafe impl<'p, I, M, O> Signal<O> for Map<'p, I, M, O>
where
	I: Lifetimed,
	O: Lifetimed,
	M: for<'v> FnMut(I::Value<'v>, MapTo<'v, O>),
{
	fn add_listener<'s>(&'s self, listener: *const dyn Pushable<O>) {
		let mut inner = self.inner.borrow_mut();
		if inner.listener.is_none() {
			self.parent.add_listener(unsafe { self.transmute_to_dyn() });
			inner.listener = Some(listener);
		}
	}
	fn remove_listener<'s>(&'s self, _listener: *const dyn Pushable<O>) {
		if let Some(_listener) = self.inner.borrow_mut().listener.take() {
			self.parent
				.remove_listener(unsafe { self.transmute_to_dyn() });
		}
	}
	fn request_fire<'s>(&'s self, ) {
		if std::mem::replace(&mut self.inner.borrow_mut().fire_requested, true) {
			self.parent.request_fire();
		}
	}
}

impl<'p, I, M, O> Pushable<I> for Map<'p, I, M, O>
where
	I: Lifetimed,
	O: Lifetimed,
	M: for<'v> FnMut(I::Value<'v>, MapTo<'v, O>),
{
	fn push<'s, 'v>(&'s self, value: <I as Lifetimed>::Value<'v>, mode: PushMode)
	where
		Self: 'v,
	{
		if let MapInner {
			listener: Some(listener),
			mapper,
			fire_requested
		} = &mut *self.inner.borrow_mut()
		{
			let fire = unsafe { &**listener };
			let next = MapTo::<'_, O> { fire, mode };
			mapper(value, next);
		}
	}
}

pub fn map<'p, S, I, M, O>(signal: &'p S, mapper: M) -> impl Signal<O> + 'p
where
	I: Lifetimed + 'p,
	O: Lifetimed + 'p,
	M: for<'v> FnMut(I::Value<'v>, MapTo<'v, O>) + 'p,
	S: ?Sized + UnsizeSignal<I>,
{
	Map {
		parent: signal.coerce(),
		inner: RefCell::new(MapInner {
			listener: Default::default(),
			mapper,
			fire_requested: false
		}),
	}
}
