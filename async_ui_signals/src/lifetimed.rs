use std::marker::PhantomData;

pub trait Lifetimed {
	type Value<'v>
	where
		Self: 'v;
}

pub struct Borrowed<T: ?Sized>(PhantomData<T>);
impl<T: ?Sized> Lifetimed for Borrowed<T> {
	type Value<'v> = &'v T where Self: 'v;
}
pub struct Owned<T>(PhantomData<T>);
impl<T> Lifetimed for Owned<T> {
	type Value<'v> = T where Self: 'v;
}

pub trait LifetimedCovariant: Lifetimed {
	fn shorten<'s, 'l: 's>(current: Self::Value<'l>) -> Self::Value<'s>
	where
		Self: 'l + 's;
}
impl<T: ?Sized> LifetimedCovariant for Borrowed<T> {
	fn shorten<'s, 'l: 's>(current: Self::Value<'l>) -> Self::Value<'s>
	where
		Self: 'l + 's,
	{
		current
	}
}
impl<T> LifetimedCovariant for Owned<T> {
	fn shorten<'s, 'l: 's>(current: Self::Value<'l>) -> Self::Value<'s>
	where
		Self: 'l + 's,
	{
		current
	}
}
