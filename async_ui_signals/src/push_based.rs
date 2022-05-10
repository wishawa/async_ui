use std::{
	cell::RefCell,
	marker::{PhantomData},
	ops::{Index, Range},
	pin::Pin,
};

trait Listenable<V>
where
	V: ?Sized,
{
	unsafe fn add_listener<'s, 'z>(&'s self, listener: &'z V)
	where
		Self: 'z;
}

trait Pushable<I> {
	fn push<'s>(&'s self, input: I);
	unsafe fn add_to_parent(self: Pin<&Self>);
	fn mount(self: Pin<&Self>) {
		async_ui_core::drop_check::check_drop_scope(self.get_ref() as *const Self as *const ());
		unsafe { self.add_to_parent() };
	}
}

struct SignalSource<T> {
	value: RefCell<T>,
	listener: RefCell<Option<*const dyn for<'x> Pushable<&'x T>>>,
}

impl<T> SignalSource<T> {
	fn new(value: T) -> Self {
		Self {
			value: RefCell::new(value),
			listener: Default::default(),
		}
	}
	fn visit_mut<F, R>(&self, visitor: F)
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
	unsafe fn add_listener<'s, 'z>(&'s self, listener: &'z V)
	where
		Self: 'z,
	{
		let coerced: *const (dyn for<'x> Pushable<&'x T> + 'v) = listener;
		let transmuted: *const (dyn for<'x> Pushable<&'x T> + 'static) =
			unsafe { std::mem::transmute(coerced) };
		*self.listener.borrow_mut() = Some(transmuted);
	}
}

trait Mapper {
	type Input<'i>
	where
		Self: 'i;
	type Output<'o>
	where
		Self: 'o;
	fn map<'m, 's>(&'s self, input: Self::Input<'m>) -> Self::Output<'m>
	where
		Self: 'm;
}

struct SignalMap<'p, M>
where
	M: Mapper + 'p,
{
	mapper: M,
	listener: RefCell<Option<*const dyn for<'x> Pushable<M::Output<'x>>>>,
	parent: &'p (dyn Listenable<Self> + 'p),
}

impl<'p, M> SignalMap<'p, M>
where
	M: Mapper + 'p,
{
	fn new<P>(mapper: M, parent: &'p P) -> Self
	where
		P: Listenable<Self> + 'p,
	{
		Self {
			mapper,
			listener: Default::default(),
			parent,
		}
	}
}

impl<'v, 'p, M, V> Listenable<V> for SignalMap<'p, M>
where
	M: Mapper + 'p,
	V: for<'x> Pushable<M::Output<'x>> + 'v,
{
	unsafe fn add_listener<'s, 'z>(&'s self, listener: &'z V)
	where
		Self: 'z,
	{
		let coerced: *const (dyn for<'x> Pushable<M::Output<'x>> + 'v) = listener;
		let transmuted: *const (dyn for<'x> Pushable<M::Output<'x>> + 'static) =
			unsafe { std::mem::transmute(coerced) };
		*self.listener.borrow_mut() = Some(transmuted);
	}
}

impl<'i, 'p, M> Pushable<M::Input<'i>> for SignalMap<'p, M>
where
	M: Mapper + 'p,
{
	fn push<'s>(&'s self, input: M::Input<'i>) {
		let output = self.mapper.map(input);
		if let Some(listener) = self.listener.borrow().as_ref() {
			let listener = unsafe { &**listener };
			listener.push(output);
		}
	}
	unsafe fn add_to_parent(self: Pin<&Self>) {
		unsafe { self.parent.add_listener(self.get_ref()) };
	}
}

struct SignalCache<'p, C> {
	cache: RefCell<Option<C>>,
	listener: RefCell<Option<*const dyn for<'x> Pushable<&'x C>>>,
	parent: &'p (dyn Listenable<Self> + 'p)
}

impl<'v, 'p, C, V> Listenable<V> for SignalCache<'p, C>
where
	V:  for<'x> Pushable<&'x C> + 'v,
{
	unsafe fn add_listener<'s, 'z>(&'s self, listener: &'z V)
	where
		Self: 'z,
	{
		let coerced: *const (dyn for<'x> Pushable<&'x C> + 'v) = listener;
		let transmuted: *const (dyn for<'x> Pushable<&'x C> + 'static) =
			unsafe { std::mem::transmute(coerced) };
		*self.listener.borrow_mut() = Some(transmuted);
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
	unsafe fn add_to_parent(self: Pin<&Self>) {
		unsafe { self.parent.add_listener(self.get_ref()) };
	}
}

fn play() {
	let s = String::from("hello world");
	let source = SignalSource {
		value: RefCell::new(s),
		listener: Default::default(),
	};

	struct SliceMapper<T: ?Sized + Index<Range<usize>>>(Range<usize>, PhantomData<T>);
	impl<T: ?Sized + Index<Range<usize>>> Mapper for SliceMapper<T> {
		type Input<'i> = &'i T where Self: 'i;
		type Output<'o> = &'o T::Output where Self: 'o;
		fn map<'x>(&self, input: Self::Input<'x>) -> Self::Output<'x> {
			&input[self.0.clone()]
		}
	}

	let mapper: SliceMapper<String> = SliceMapper(1..5, PhantomData);

	let mapped = SignalMap::new(mapper, &source);

	let mapper2: SliceMapper<str> = SliceMapper(1..3, PhantomData);
	let mapped2 = SignalMap::new(mapper2, &mapped);

}

#[cfg(test)]
mod tests {

}