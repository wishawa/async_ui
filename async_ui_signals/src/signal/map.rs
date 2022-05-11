use std::cell::RefCell;

use crate::{ext::UnsizeSignal, lifetimed::Lifetimed};

use super::{Pushable, Signal};

pub struct PushNext<'f, I>
where
	I: Lifetimed,
{
	fire: &'f dyn Pushable<I>,
}
impl<'f, I> PushNext<'f, I>
where
	I: Lifetimed,
{
	pub fn push<'v>(self, value: I::Value<'v>)
	where
		'f: 'v,
	{
		self.fire.push(value);
	}
}

pub struct Map<'p, I, M, O>
where
	I: Lifetimed,
	O: Lifetimed,
	M: for<'v> FnMut(I::Value<'v>, PushNext<'v, O>),
{
	parent: &'p (dyn Signal<I> + 'p),
	inner: RefCell<MapInner<M, O>>,
}

impl<'p, I, M, O> Map<'p, I, M, O>
where
	I: Lifetimed,
	O: Lifetimed,
	M: for<'v> FnMut(I::Value<'v>, PushNext<'v, O>),
{
	unsafe fn transmute_to_dyn(&self) -> *const dyn Pushable<I> {
		let coerced: *const (dyn Pushable<I> + '_) = self;
		let transmuted: *const (dyn Pushable<I> + 'static) =
			unsafe { std::mem::transmute(coerced) };
		transmuted
	}
}

struct MapInner<M, O>
where
	O: Lifetimed,
{
	listener: Option<*const dyn Pushable<O>>,
	mapper: M,
}

unsafe impl<'p, I, M, O> Signal<O> for Map<'p, I, M, O>
where
	I: Lifetimed,
	O: Lifetimed,
	M: for<'v> FnMut(I::Value<'v>, PushNext<'v, O>),
{
	fn add_listener<'s>(&'s self, listener: *const dyn Pushable<O>) {
		let mut bm = self.inner.borrow_mut();
		if bm.listener.is_none() {
			self.parent.add_listener(unsafe { self.transmute_to_dyn() });
			bm.listener = Some(listener);
		}
	}
	fn remove_listener<'s>(&'s self, _listener: *const dyn Pushable<O>) {
		if let Some(_listener) = self.inner.borrow_mut().listener.take() {
			self.parent
				.remove_listener(unsafe { self.transmute_to_dyn() });
		}
	}
}

impl<'p, I, M, O> Pushable<I> for Map<'p, I, M, O>
where
	I: Lifetimed,
	O: Lifetimed,
	M: for<'v> FnMut(I::Value<'v>, PushNext<'v, O>),
{
	fn push<'s, 'v>(&'s self, value: <I as Lifetimed>::Value<'v>)
	where
		Self: 'v,
	{
		if let MapInner {
			listener: Some(listener),
			mapper,
		} = &mut *self.inner.borrow_mut()
		{
			let fire = unsafe { &**listener };
			let next = PushNext::<'_, O> { fire };
			mapper(value, next);
		}
	}
}

pub fn map<'p, S, I, M, O>(signal: &'p S, mapper: M) -> impl Signal<O> + 'p
where
	I: Lifetimed + 'p,
	O: Lifetimed + 'p,
	M: for<'v> FnMut(I::Value<'v>, PushNext<'v, O>) + 'p,
	S: ?Sized + UnsizeSignal<I>,
{
	Map {
		parent: signal.coerce(),
		inner: RefCell::new(MapInner {
			listener: Default::default(),
			mapper,
		}),
	}
}
